FROM simplefrag AS simplefrag

FROM python:3.8

RUN python -m pip install --upgrade pip
RUN python -m pip install streamlit pandas

WORKDIR /work

COPY --from=simplefrag /app/simple-fragment-server /app/simple-fragment-server

ENTRYPOINT [ "/app/simple-fragment-server", "--command", "streamlit run app.py --server.port 9090" ]
