FROM python:3.14.1-alpine3.21
HEALTHCHECK CMD "belt --version"
RUN adduser -h /app -D belt
USER belt
COPY . /app
ENV PATH="/app/.local/bin:/usr/local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
RUN /usr/local/bin/pip install --user --no-cache-dir pipx==1.7.1 && \
  /app/.local/bin/pipx install /app
ENTRYPOINT ["belt"]
