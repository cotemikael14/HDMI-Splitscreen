import cv2
from threading import Thread

class VideoCapture:
    def __init__(self, source=0):
        self.source = source
        self.cap = cv2.VideoCapture(source)
        self.ret = False
        self.frame = None
        self.stopped = False
        self.thread = None

    def start(self):
        """Start the video capture in a new thread."""
        self.stopped = False
        self.thread = Thread(target=self.update)
        self.thread.start()
        return self

    def update(self):
        """Continuously capture frames from the video source."""
        while not self.stopped:
            self.ret, self.frame = self.cap.read()

    def read(self):
        """Return the latest captured frame."""
        return self.ret, self.frame

    def stop(self):
        """Stop the video capture and release resources."""
        self.stopped = True
        if self.thread:
            self.thread.join()
        self.cap.release()
