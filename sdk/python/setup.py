from setuptools import setup

setup(
    name="freeco",
    version="0.1.0",
    description="Official Python client for the Freeco Agent OS REST API",
    py_modules=["freeco_sdk", "freeco_client"],
    python_requires=">=3.8",
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
)
