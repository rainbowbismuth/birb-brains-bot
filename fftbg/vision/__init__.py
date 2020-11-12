from imutils import contours
import imutils
import cv2
import numpy as np


# A command line to store images...
#
# ffmpeg -i $(streamlink --stream-url https://www.twitch.tv/fftbattleground best)
#   -filter:v "crop=990:740:145:260" -qscale:v 4 -r 5 -f image2 output_%05d.jpg


VITAL_LOCATIONS = {
    'minHP': (350, 585, 60, 28),
    'maxHP': (350+75, 585+15, 60, 28),
    'minMP': (350, 620, 60, 28),
    'maxMP': (350+75, 620+15, 60, 28),
    'minCT': (350, 655, 60, 28),
    'maxCT': (350+75, 655+15, 60, 28),
    'brave': (725, 653, 42, 30),
    'faith': (877, 653, 42, 30)
}


def thresh_inv(img):
    return 255 - cv2.threshold(img, 125, 255, cv2.THRESH_BINARY_INV, cv2.THRESH_OTSU)[1]


def thresh_dark(img):
    return 255 - cv2.threshold(img, 100, 255, cv2.THRESH_BINARY, cv2.THRESH_OTSU)[1]


VITAL_THRESH = {
    'minHP': thresh_inv,
    'maxHP': thresh_inv,
    'minMP': thresh_inv,
    'maxMP': thresh_inv,
    'minCT': thresh_inv,
    'maxCT': thresh_inv,
    'brave': thresh_dark,
    'faith': thresh_dark
}

# image = cv2.imread("watch-stream/output_00560.jpg")
# image = cv2.imread("watch-stream/output_00592.jpg")
image = cv2.imread("watch-stream/output_00583.jpg")
gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)


def crop(img, x, y, width, height):
    return img[y:y+height, x:x+width]


def crop_vital(img, vital):
    (x, y, width, height) = VITAL_LOCATIONS[vital]
    return crop(img, x, y, width, height)


def thresh_vital(img, vital):
    return VITAL_THRESH[vital](crop_vital(img, vital))


def crop_name(img):
    return crop(img, 605, 545, 325, 40)


def thresh_name(img):
    return thresh_dark(crop_name(img))


def crop_job(img):
    return crop(img, 605, 595, 325, 40)


def thresh_job(img):
    return thresh_dark(crop_job(img))


def crop_ability(img):
    return crop(img, 270, 122, 425, 58)


def thresh_ability(img):
    return thresh_dark(crop_ability(img))

    # img = 255 - thresh_dark(crop(img, 270, 122, 425, 58))
    # return cv2.erode(img, cv2.getStructuringElement(cv2.MORPH_CROSS, (3, 3)), iterations=1)


for vital in VITAL_LOCATIONS.keys():
    thresh = thresh_vital(gray, vital)
    cv2.imwrite(f'hello{vital}.png', thresh)


def number_rects(orig, img):
    ctrs, hier = cv2.findContours(img.copy(), cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    rects = [cv2.boundingRect(ctr) for ctr in ctrs]
    out = orig.copy()

    # Sort rects left to right for "reading" order
    rects.sort(key=lambda rect: rect[0])

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
        return out

    # Find the minimum y so all letters can start there
    min_y = min([rect[1] for rect in filtered_rects])

    prev_x = min([rect[0] for rect in filtered_rects])
    for (x, y, w, h) in filtered_rects:

        # If a letter is too far away it is not part of the word or phrase, so we stop there
        if x - prev_x > 60:
            break
        prev_x = x

        # Adjust our bounds so that all rects start at min_y
        diff_y = min_y - y
        y = int(min_y)
        h -= int(diff_y)

        cv2.rectangle(out, (x, y), (x + w, y + h), (0, 255, 0), 1)
    return out


thresh = cv2.threshold(gray, 125, 255, cv2.THRESH_BINARY_INV, cv2.THRESH_OTSU)[1]

blurred = cv2.GaussianBlur(gray, (5, 5), 0)
edged = cv2.Canny(blurred, 100, 200, 255)

cv2.imwrite("hello.png", edged)
cv2.imwrite("name.png", thresh_name(gray))
cv2.imwrite("ability.png", thresh_ability(gray))
cv2.imwrite("show_rects.png", number_rects(crop_vital(image, 'brave'), thresh_vital(gray, 'brave')))
cv2.imwrite("show_rects2.png", number_rects(crop_ability(image), thresh_ability(gray)))
cv2.imwrite("show_rects3.png", number_rects(crop_name(image), thresh_name(gray)))
cv2.imwrite("show_rects4.png", number_rects(crop_job(image), thresh_job(gray)))