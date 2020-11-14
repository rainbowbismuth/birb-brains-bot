import difflib
import json
from pathlib import Path

import cv2
from termcolor import colored

import fftbg.vision as vision

TEST_RESULTS = []
READER = vision.default_character_reader()

OK_DOT = colored('.', 'green')
FAIL_DOT = colored('.', 'red')
WRAP_AT = 100


def add_result(img, ok, expected, reading):
    TEST_RESULTS.append({"file": img, "ok": ok, "expected": expected, "reading": reading, "idx": len(TEST_RESULTS)})
    newline = ''
    if len(TEST_RESULTS) % WRAP_AT == (WRAP_AT - 1):
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
            c = s[-1]
            if s[-1] == ' ':
                c = '_'
            res.append(colored(c, 'green'))
        elif s[0] == '+':
            c = s[-1]
            if s[-1] == ' ':
                c = '_'
            res.append(colored(c, 'red'))
    return ''.join(res)


def report_results():
    print('\n')
    h = ["file", "finder", "expected", "actual", "diff"]
    print(colored(f'{h[0]:<38} {h[1]:<10} {h[2]:<30} {h[3]:<30} {h[4]}', 'cyan'))
    print(f'{"-" * 37}  {"-" * 9}  {"-" * 29}  {"-" * 29}  {"-" * 29}')
    for result in TEST_RESULTS:
        if not result["ok"]:
            expected = result["expected"]
            actual = result["reading"].value
            name = result["reading"].name
            diff = diff_string(expected, actual)
            print(f'{result["file"]:<38} {name:<10} {expected:<30} {actual:<30} {diff}')

    ok_count = len([r for r in TEST_RESULTS if r["ok"]])
    print(f'\n{ok_count} OK / {len(TEST_RESULTS)} TOTAL')


def run_tests():
    run_text()
    print()


if __name__ == '__main__':
    run_tests()
    report_results()
