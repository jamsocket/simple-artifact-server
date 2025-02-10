#!/usr/bin/env python3
import argparse
from openai import OpenAI
from jamsocket.client import Client as JamsocketClient

client = OpenAI()
import os
import sys


def parse_output(output: str) -> str:
    result = []
    in_code = False
    lines = iter(output.split("\n"))
    for line in lines:
        if line.strip().startswith("```"):
            in_code = not in_code
        elif in_code:
            result.append(line)

    return "\n".join(result)


def generate_streamlit_app(prompt: str, output_file: str, model: str = "gpt-4", temperature: float = 0.0) -> None:
    """Generate a Streamlit application based on the given prompt and upload it to Jamsocket.

    Args:
        prompt (str): The prompt describing the desired Streamlit application
        output_file (str): Name to use for the uploaded artifact
        model (str, optional): OpenAI model to use. Defaults to "gpt-4"
        temperature (float, optional): Sampling temperature. Defaults to 0.0
    """
    if not os.getenv("OPENAI_API_KEY"):
        print("Error: Please set your OpenAI API key as OPENAI_API_KEY environment variable")
        sys.exit(1)

    try:
        # Initialize Jamsocket client
        jamsocket = JamsocketClient()

        # Start a backend with the streamlit-artifacts service
        backend = jamsocket.start_backend("streamlit-artifacts")
        print("Waiting for backend to be ready...")
        backend.wait_ready()

        # Generate the Streamlit app code
        response = client.chat.completions.create(
            model=model,
            messages=[
            {"role": "system", "content": "You are a helpful assistant that generates Streamlit applications. Generate a complete, working Streamlit application in Python that addresses the user's request. Include all necessary imports and ensure the code is properly formatted and documented. Return only a single markdown code block with no commentary."},
            {"role": "user", "content": prompt}
        ],
        temperature=temperature)

        generated_code = response.choices[0].message.content
        generated_code = parse_output(generated_code)

        # Upload the generated code to the backend
        print(f"Uploading {output_file} to Jamsocket backend...")
        # Upload the code as an artifact
        backend.upload_artifact(output_file, generated_code.encode())
        print(f"Successfully uploaded {output_file} to Jamsocket backend {backend.id}")

        print(f"Successfully generated Streamlit application: {output_file}")
        print("\nTo run the application:")
        print(f"streamlit run {output_file}")

    except Exception as e:
        print(f"Error generating Streamlit application: {str(e)}")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description='Generate a Streamlit application from a prompt using OpenAI API')
    parser.add_argument('prompt', help='Prompt describing the desired Streamlit application')
    parser.add_argument('-o', '--output', default='app.py',
                        help='Output file path (default: app.py)')
    parser.add_argument('-m', '--model', default='gpt-4',
                        help='OpenAI model to use (default: gpt-4)')
    parser.add_argument('-t', '--temperature', type=float, default=0.0,
                        help='Sampling temperature (default: 0.0)')

    args = parser.parse_args()
    generate_streamlit_app(args.prompt, args.output, args.model, args.temperature)

if __name__ == "__main__":
    main()
