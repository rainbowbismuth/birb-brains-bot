FROM python:3.8
RUN apt-get update && apt-get -y upgrade
COPY ./fftbg app/fftbg
COPY ./data/static app/data/static
COPY ./requirements.txt app/
WORKDIR app
RUN pip install --no-cache-dir -r requirements.txt
CMD ["python", "-m", "fftbg.jobs"]
