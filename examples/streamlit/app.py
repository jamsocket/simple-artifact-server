import streamlit as st
import numpy as np
import matplotlib.pyplot as plt

def mandelbrot(c, max_iter):
    z = c
    for n in range(max_iter):
        if abs(z) > 2:
            return n
        z = z*z + c
    return max_iter

def draw_mandelbrot(xmin,xmax,ymin,ymax,width,height,max_iter):
    r1 = np.linspace(xmin, xmax, width)
    r2 = np.linspace(ymin, ymax, height)
    return (r1,r2,np.array([[mandelbrot(complex(r, i),max_iter) for r in r1] for i in r2]))

def main():
    st.title('Mandelbrot Explorer')

    # Set up the parameters
    xmin = st.sidebar.slider('xmin', -2.0, 0.5, -2.0)
    xmax = st.sidebar.slider('xmax', -2.0, 0.5, 0.5)
    ymin = st.sidebar.slider('ymin', -1.5, 1.5, -1.5)
    ymax = st.sidebar.slider('ymax', -1.5, 1.5, 1.5)
    width = st.sidebar.slider('width', 100, 1000, 500)
    height = st.sidebar.slider('height', 100, 1000, 500)
    max_iter = st.sidebar.slider('max_iter', 1, 1000, 256)

    # Draw the mandelbrot set
    d = draw_mandelbrot(xmin,xmax,ymin,ymax,width,height,max_iter)
    st.pyplot(plt.imshow(d[2], extent=(xmin, xmax, ymin, ymax)))

if __name__ == "__main__":
    main()