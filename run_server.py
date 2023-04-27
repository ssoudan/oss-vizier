from typing import Sequence

from absl import app
from absl import flags
from absl import logging

from concurrent import futures
import datetime
import time

import attr
import grpc

from vizier.service import datastore
from vizier.service import constants
from vizier.service import stubs_util
from vizier.service import vizier_service
from vizier.service import vizier_service_pb2_grpc


@attr.define
class VizierService:
    """Vizier service which runs Pythia and Vizier Server in the same process.

    Both servers have access to the others' literal class instances.
    """
    _host: str = attr.field(init=True, default='0.0.0.0')
    _database_url: str = attr.field(
        init=True, default=constants.SQL_MEMORY_URL, kw_only=True)
    _early_stop_recycle_period: datetime.timedelta = attr.field(
        init=False, default=datetime.timedelta(seconds=0.1))
    _port: int = attr.field(init=True, default=28080)
    _servicer: vizier_service.VizierServicer = attr.field(init=False)
    _server: grpc.Server = attr.field(init=False)
    stub: vizier_service_pb2_grpc.VizierServiceStub = attr.field(init=False)

    @property
    def datastore(self) -> datastore.DataStore:
        return self._servicer.datastore

    @property
    def endpoint(self) -> str:
        return f'{self._host}:{self._port}'

    def __attrs_post_init__(self):
        # Setup Vizier server.
        self._servicer = vizier_service.VizierServicer(
            database_url=self._database_url,
            early_stop_recycle_period=self._early_stop_recycle_period)
        self._server = grpc.server(futures.ThreadPoolExecutor(max_workers=30))
        vizier_service_pb2_grpc.add_VizierServiceServicer_to_server(
            self._servicer, self._server)
        self._server.add_insecure_port(self.endpoint)
        # self._server.add_secure_port(
        # self.endpoint, grpc.local_server_credentials())
        self._server.start()
        self.stub = stubs_util.create_vizier_server_stub(self.endpoint)

    def wait_for_early_stop_recycle_period(self) -> None:
        time.sleep(self._early_stop_recycle_period.total_seconds())


flags.DEFINE_string(
    'host', '0.0.0.0',
    'Host location for the server. For distributed cases, use the IP address.')

flags.DEFINE_integer(
    'port', 28080,
    'Port to listen to. 28080 by default.'
)

FLAGS = flags.FLAGS

_ONE_DAY_IN_SECONDS = 60 * 60 * 24


def main(argv: Sequence[str]) -> None:
    if len(argv) > 1:
        raise app.UsageError('Too many command-line arguments.')

    service = VizierService(host=FLAGS.host, port=FLAGS.port)
    logging.info('Address to Vizier Server is: %s', service.endpoint)

    # prevent the main thread from exiting
    try:
        while True:
            time.sleep(_ONE_DAY_IN_SECONDS)
    except KeyboardInterrupt:
        del service


if __name__ == '__main__':
    app.run(main)
