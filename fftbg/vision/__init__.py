import time
from pathlib import Path

import cv2
import numpy as np
import tensorflow as tf
from sklearn.cluster import MiniBatchKMeans
from sklearn.model_selection import train_test_split


# A command line to store images...
#
# ffmpeg -i $(streamlink --stream-url https://www.twitch.tv/fftbattleground best)
#   -filter:v "crop=990:740:145:260" -qscale:v 4 -r 6 -f image2 output_%05d.jpg

# Download a stream video to replay later
#
# ffmpeg -i $(streamlink --stream-url https://www.twitch.tv/fftbattleground best)
#   -filter:v "crop=990:740:145:260" -vcodec libx264 -crf 32 -an -r 6 stream.mp4

# Create a RAM disk on Mac OS
#
# diskutil erasevolume HFS+ RAM_Disk_512MB $(hdiutil attach -nomount ram://512000)


def crop(img, x, y, width, height):
    return img[y:y + height, x:x + width]


def crop_rect(img, rect):
    x, y, width, height = rect
    return img[y:y + height, x:x + width]


def find_characters(img):
    ctrs, hier = cv2.findContours(img.copy(), cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    rects = [cv2.boundingRect(ctr) for ctr in ctrs]

    # Filter out rects that are too small or large to be letters
    filtered_rects = []
    for (x, y, w, h) in rects:
        if w < 5 or h < 9:
            continue
        if w > 30 or h > 40:
            continue
        filtered_rects.append((x, y, w, h))

    # Stop early if no letters/numbers
    if not filtered_rects:
        return []

    median_x = np.median([rect[0] for rect in filtered_rects])
    rects_with_distance = [(abs(median_x - x), x, y, w, h) for (x, y, w, h) in filtered_rects]

    # Sort rects by distance from median, so closest is first
    rects_with_distance.sort(key=lambda rect: rect[0])

    filtered_rects = []
    prev_dist = rects_with_distance[0][0]
    for (dist, x, y, w, h) in rects_with_distance:
        if dist - prev_dist > 45:
            break
        prev_dist = dist
        filtered_rects.append((x, y, w, h))

    # Sort from left-right (reading order)
    filtered_rects.sort(key=lambda rect: rect[0])

    out = []

    # Find the minimum y so all letters can start there
    min_y = min([rect[1] for rect in filtered_rects])

    for (x, y, w, h) in filtered_rects:
        # Adjust our bounds so that all rects start at min_y
        diff_y = min_y - y
        y = int(min_y)
        h -= int(diff_y)
        out.append((max(x - 1, 0), max(y - 1, 0), w + 2, h + 2))

    return out


CHARSET = "?01234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ+"


def load_labelled_digits():
    xs = []
    ys = []

    for path in Path('data/labelled').iterdir():
        if path.name[0] == '.':
            continue
        char = path.name[-1]
        index = CHARSET.index(char)
        for image_path in path.glob('*.png'):
            image = cv2.imread(image_path.as_posix())
            gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
            xs.append(gray)
            ys.append(index)

    return np.array(xs), np.array(ys)


def train_charset():
    model = tf.keras.models.Sequential([
        tf.keras.layers.Reshape((32, 32, 1)),
        tf.keras.layers.Conv2D(filters=6, kernel_size=(3, 3), activation='relu', input_shape=(32, 32, 1)),
        tf.keras.layers.AveragePooling2D(),
        tf.keras.layers.Conv2D(filters=16, kernel_size=(3, 3), activation='relu'),
        tf.keras.layers.AveragePooling2D(),
        tf.keras.layers.Flatten(),
        tf.keras.layers.Dense(256, activation='relu'),
        tf.keras.layers.Dense(128, activation='relu'),
        tf.keras.layers.Dense(len(CHARSET),
                              activation='softmax',
                              kernel_initializer='he_normal',
                              kernel_regularizer=tf.keras.regularizers.l2(0.01))
    ])

    model.compile(
        optimizer='adam',
        loss='sparse_categorical_crossentropy',
        metrics=['accuracy'],
    )

    xs, ys = load_labelled_digits()
    xs = xs / 255.0
    X_train, X_test, y_train, y_test = train_test_split(xs, ys, test_size=0.2)

    early_stopping_cb = tf.keras.callbacks.EarlyStopping(
        patience=10, monitor='val_loss', restore_best_weights=True)

    print(X_train.shape)
    model.fit(X_train, y_train, epochs=200, validation_split=0.2, callbacks=[early_stopping_cb])
    model.evaluate(X_test, y_test, verbose=2)
    model.save('data/charset_model.h5')


# train_charset()


class Frame:
    def __init__(self, color_image):
        self.color_image = color_image
        self.gray = cv2.cvtColor(color_image, cv2.COLOR_BGR2GRAY)
        self.gray_min = np.min(color_image, axis=2)
        self.gray_max = np.max(color_image, axis=2)


class RectangleFinder:
    def __init__(self, name: str, rect, prepare_fn, found_fn):
        self.name = name
        self.rect = rect
        self.prepare_fn = prepare_fn
        self.found_fn = found_fn

    def find_rects(self, frame: Frame):
        prepared_img = self.prepare_fn(frame, self.rect)
        rects = find_characters(prepared_img)
        found_imgs = [self.found_fn(prepared_img, rect) for rect in rects]
        abs_rects = [(x + self.rect[0], y + self.rect[1], w, h) for (x, y, w, h) in rects]
        return list(zip(found_imgs, abs_rects))


def char_found(prepared_img, rect):
    cropped = crop_rect(prepared_img, rect)
    return cv2.resize(cropped, (32, 32))


def light_text(frame: Frame, rect):
    cropped = crop_rect(frame.gray_min, rect)
    return cv2.threshold(cropped, 125, 255, cv2.THRESH_BINARY)[1]


def dark_text(frame: Frame, rect):
    cropped = crop_rect(frame.gray_max, rect)
    return cv2.threshold(cropped, 100, 255, cv2.THRESH_BINARY_INV)[1]


class CharacterReader:
    def __init__(self, char_model):
        self.char_model = char_model

    def _read_char_model(self, characters):
        if characters is None or len(characters) == 0:
            return []
        y_pred = self.char_model(np.array([x / 255.0 for x in characters]))
        chars = [CHARSET[i] for i in np.argmax(y_pred, axis=1)]
        certainty = np.max(y_pred, axis=1)
        return list(zip(certainty, chars))

    def read_digits(self, characters):
        return self._read_char_model(characters)

    def read_characters(self, characters):
        return self._read_char_model(characters)


class VitalReading:
    def __init__(self, finder: RectangleFinder, prob_chars, rects, images):
        self.name = finder.name
        self.finder = finder
        self.prob_chars = prob_chars
        self.value = ''.join(c for (_, c) in self.prob_chars)
        self.certainty = np.product(p for (p, _) in self.prob_chars)
        self.rects = rects
        self.images = images


FINDERS = [
    RectangleFinder('minHP', rect=(350, 588, 60, 27), prepare_fn=light_text, found_fn=char_found),
    RectangleFinder('maxHP', rect=(423, 601, 60, 27), prepare_fn=light_text, found_fn=char_found),
    RectangleFinder('minMP', rect=(350, 623, 60, 27), prepare_fn=light_text, found_fn=char_found),
    RectangleFinder('maxMP', rect=(423, 636, 60, 27), prepare_fn=light_text, found_fn=char_found),
    RectangleFinder('minCT', rect=(350, 658, 60, 27), prepare_fn=light_text, found_fn=char_found),
    RectangleFinder('maxCT', rect=(423, 671, 60, 27), prepare_fn=light_text, found_fn=char_found),
    RectangleFinder('brave', rect=(725, 653, 42, 30), prepare_fn=dark_text, found_fn=char_found),
    RectangleFinder('faith', rect=(877, 653, 42, 30), prepare_fn=dark_text, found_fn=char_found),
    RectangleFinder('name', rect=(610, 545, 320, 40), prepare_fn=dark_text, found_fn=char_found),
    RectangleFinder('job', rect=(610, 595, 320, 40), prepare_fn=dark_text, found_fn=char_found),
    RectangleFinder('ability', rect=(270, 122, 425, 58), prepare_fn=dark_text, found_fn=char_found),
]


def read_vital_new(frame: Frame, reader: CharacterReader, finder: RectangleFinder) -> VitalReading:
    img_rects = finder.find_rects(frame)
    rects = [rect for (_, rect) in img_rects]
    imgs = [img for (img, _) in img_rects]
    prob_chars = reader.read_characters(imgs)
    return VitalReading(finder, prob_chars, rects, imgs)


def cluster_images():
    from tqdm import tqdm
    kmeans = MiniBatchKMeans(n_clusters=100)

    images = []
    for path in tqdm(list(Path('letters').glob('*.png'))):
        image = cv2.imread(path.as_posix())
        gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
        images.append((path.name, gray.flatten()))

    kmeans.partial_fit([img for (path, img) in images])

    predicted = kmeans.predict([img for (path, img) in images])

    for i, (path, image) in enumerate(tqdm(images)):
        reshaped = image.reshape((32, 32))
        bucket = predicted[i]
        Path(f'clustered/{bucket}/').mkdir(parents=True, exist_ok=True)
        cv2.imwrite(f'clustered/{bucket}/{i:04d}_{path}', reshaped)


# cluster_images()


def py_gui():
    import pygame
    import sys
    pygame.init()
    pygame.font.init()
    font = pygame.font.Font('fftbg/vision/RobotoCondensed-Regular.ttf', 20)
    pygame.display.set_caption("Birb Brains Vision")
    pygame.display.set_icon(pygame.image.load('fftbg/vision/icon.png'))
    size = width, height = 990, 740 + 200
    screen = pygame.display.set_mode(size)
    black = 0, 0, 0
    reader = CharacterReader(tf.keras.models.load_model('data/charset_model.h5'))

    surface = pygame.Surface((990, 740))

    offsets = [(5, i * 28 + 5 + 740) for i in range(6)] + [(305, i * 28 + 5 + 740) for i in range(6)]
    finder_names = [font.render(finder.name, True, (255, 255, 255)) for finder in FINDERS]

    while True:
        paths = sorted(Path('/Volumes/RAM_Disk_512MB/').glob('*.jpg'))

        for path_i, path in enumerate(paths):
            f_start = time.monotonic()

            image = None
            if path_i < 10:
                image = cv2.imread(path.as_posix())
            path.unlink(missing_ok=True)
            if image is None:
                continue

            frame = Frame(image)

            color_mapped = cv2.applyColorMap(frame.gray, cv2.COLORMAP_BONE)

            for i, finder in enumerate(FINDERS):
                reading = read_vital_new(frame, reader, finder)
                if not reading.rects:
                    continue
                (x, y, w, h) = finder.rect
                cv2.rectangle(color_mapped, (x, y), (x + w, y + h), (0, 255, 0), 1)
                for (x, y, w, h) in reading.rects:
                    cv2.rectangle(color_mapped, (x, y), (x + w, y + h), (0, 0, 255), 1)

                offset = offset_x, offset_y = offsets[i]
                screen.blit(finder_names[i], offset)
                text_surf = font.render(reading.value, True, (255, 255, 255))
                screen.blit(text_surf, (offset_x + 100, offset_y))

            color_mapped = color_mapped[..., ::-1].copy()
            arr = pygame.surfarray.map_array(surface, color_mapped).swapaxes(0, 1)
            pygame.surfarray.blit_array(surface, arr)
            screen.blit(surface, surface.get_rect())
            pygame.display.flip()
            screen.fill(black)

            f_duration = time.monotonic() - f_start
            print(f'{len(paths):03d}', f'{f_duration:2f}', path.name)

            pygame.time.delay(max(0, int(75 - f_duration * 1000)))

            for event in pygame.event.get():
                if event.type == pygame.QUIT:
                    sys.exit()
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                sys.exit()


py_gui()
