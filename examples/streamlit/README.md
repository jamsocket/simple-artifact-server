Streamlit demo
==============

Upload to Jamsocket:

```bash
npx jamsocket@latest service create my-streamlit-demo
npx jamsocket@latest service push my-streamlit-demo -f ./Dockerfile
```

Spawn an instance of the service:

```bash
npx jamsocket@latest spawn my-streamlit-demo
```

Open the "ready" URL in your browser.

Then, upload a streamlit app:

```bash
curl -X POST --data-binary @app.py https://<result of spawn>/_frag/upload/app.py
```

## Customizing

You can customize the server by modifying the `Dockerfile`, for example, to add Python dependencies.
