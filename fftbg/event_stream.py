import copy
import logging
from typing import List, Tuple

from walrus import Database, Stream

import fftbg.server

LOG = logging.getLogger(__name__)
EVENT_STREAM = 'event_stream'


class EventStream:
    def __init__(self, db: Database):
        self.stream: Stream = db.Stream(EVENT_STREAM)
        self.sender_tag: str = fftbg.server.get_name()
        self.last_id = '$'
        messages = self.stream.revrange('+', '-', 1)
        if messages:
            self.last_id = messages[-1][0]

    def publish(self, msg: dict):
        msg = copy.copy(msg)
        msg['sender'] = self.sender_tag
        self.stream.add(msg)

    def read(self, count: int = 1000, block: int = 0) -> List[Tuple[bytes, dict]]:
        messages = self.stream.read(count=count, block=block, last_id=self.last_id)
        if messages:
            self.last_id = messages[-1][0]
        return messages
