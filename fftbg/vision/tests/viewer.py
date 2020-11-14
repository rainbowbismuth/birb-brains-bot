import cv2
from flask import Flask, Response, render_template, send_from_directory

import fftbg.vision as vision
import fftbg.vision.tests.runner as runner

app = Flask(__name__, static_url_path='')
app.config['TEMPLATES_AUTO_RELOAD'] = True

runner.run_tests()


def to_png(image):
    return Response(cv2.imencode('.png', image)[1].tobytes(), mimetype='image/png')


@app.route('/')
def show_index():
    failures = [result for result in runner.TEST_RESULTS if not result["ok"]]
    return render_template('index.html', failures=failures)


@app.route('/test/<int:index>')
def show_test(index):
    result = runner.TEST_RESULTS[index]
    return render_template('test.html', result=result)


@app.route('/test/<int:index>/char/<int:char_idx>')
def show_test_char(index, char_idx):
    result = runner.TEST_RESULTS[index]
    image = result["reading"].images[char_idx]
    return to_png(image)


@app.route('/test/<int:index>/prepared')
def show_test_prepared(index):
    result = runner.TEST_RESULTS[index]
    image = result["reading"].notes["prepared"]
    return to_png(image)


@app.route('/test/<int:index>/prepared-with-rects')
def show_test_prepared_rects(index):
    result = runner.TEST_RESULTS[index]
    image = result["reading"].notes["prepared"]
    color_mapped = cv2.applyColorMap(image, cv2.COLORMAP_BONE)
    vision.add_reading_rects_cropped(color_mapped, result["reading"])
    return to_png(color_mapped)


@app.route('/static/<path:path>')
def show_static(path):
    return send_from_directory('', path)
