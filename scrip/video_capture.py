import cv2
from threading import Thread

class VideoCapture:
    def __init__(self, source=0):
        self.source = source
        self.cap = cv2.VideoCapture(source)
        # Réglez la résolution pour améliorer les performances
        self.cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
        self.cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)

        self.ret = False
        self.frame = None
        self.stopped = False
        self.thread = None

    def start(self):
        """Start the video capture in a new thread."""
        self.stopped = False
        self.thread = Thread(target=self.update, daemon=True)
        self.thread.start()
        return self

    def update(self):
        """Continuously capture frames from the video source."""
        while not self.stopped:
            # Lire uniquement la dernière image dans le buffer pour réduire le lag
            self.cap.grab()
            self.ret, self.frame = self.cap.retrieve()

    def read(self):
        """Return the latest captured frame."""
        return self.ret, self.frame

    def stop(self):
        """Stop the video capture and release resources."""
        self.stopped = True
        if self.thread:
            self.thread.join()
        self.cap.release()

# Exemple d'utilisation
if __name__ == "__main__":
    video_stream = VideoCapture(0).start()

    while True:
        ret, frame = video_stream.read()
        if not ret:
            continue

        cv2.imshow("Video Stream", frame)

        # Arrêter la capture si 'q' est pressé
        if cv2.waitKey(1) & 0xFF == ord('q'):
            break

    video_stream.stop()
    cv2.destroyAllWindows()
