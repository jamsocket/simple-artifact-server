FROM ghcr.io/jamsocket/simple-artifact-server:latest AS simplefrag

FROM python:3.8

RUN python -m pip install --upgrade pip
RUN python -m pip install streamlit pandas matplotlib numpy

WORKDIR /work

COPY --from=simplefrag /app/simple-artifact-server /app/simple-artifact-server

ENTRYPOINT [ "/app/simple-artifact-server", "--command", "streamlit run app.py --server.port 9090" ]
