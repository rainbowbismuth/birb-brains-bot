import os


def progress_bar(iterable):
    if int(os.environ.get('PROGRESS_BAR', 0)):
        import tqdm
        return tqdm.tqdm(iterable)
    else:
        return iterable
