import fftbg.vision as vision
import json
from pathlib import Path
import cv2
from termcolor import colored
import difflib

TEST_RESULTS = []
READER = vision.default_character_reader()

OK_DOT = colored('.', 'green')
FAIL_DOT = colored('.', 'red')
WRAP_AT = 90


def add_result(img, ok, expected, reading):
    TEST_RESULTS.append({"img": img, "ok": ok, "expected": expected, "reading": reading})
    newline = ''
    if len(TEST_RESULTS) % WRAP_AT == (WRAP_AT-1):
        newline = '\n'
    if ok:
        print(OK_DOT, end=newline)
    else:
        print(FAIL_DOT, end=newline)


def run_text():
    test_cases = json.loads(Path('fftbg/vision/tests/text.json').read_text())

    for fp, case in test_cases.items():
        img = cv2.imread('fftbg/vision/tests/' + fp)
        frame = vision.Frame(img)

        for key, value in case.items():
            finder = vision.FINDERS[key]
            reading = vision.read_vital_new(frame, READER, finder)
            ok = reading.value == value
            add_result(fp, ok, value, reading)


def diff_string(expected, actual):
    res = []
    for s in difflib.ndiff(expected, actual):
        if s[0] == ' ':
            res.append(s[-1])
        elif s[0] == '-':
            res.append(colored(s[-1], 'green'))
        elif s[0] == '+':
            res.append(colored(s[-1], 'red'))
    return ''.join(res)


def report_results():
    print()
    for result in TEST_RESULTS:
        if not result["ok"]:
            expected = result["expected"]
            actual = result["reading"].value
            name = result["reading"].name
            diff = diff_string(expected, actual)
            print(f'{result["img"]:<30} {name:<10} {expected:<30} {actual:<30} {diff}')


run_text()
report_results()
