import pygame
from .config import *


button_bounds_w = STEP_BUTTON_BOUNDING_BOX.width / 8
button_bounds_h = STEP_BUTTON_BOUNDING_BOX.height / 2
button_w = button_bounds_w * 0.75
button_h = button_bounds_h * 0.75


def draw_a_button(i, button_grid_x, button_grid_y, selected: bool, playing: bool):
    center_x = button_bounds_w * button_grid_x + \
        button_bounds_w / 2 + STEP_BUTTON_BOUNDING_BOX.left
    center_y = button_bounds_h * button_grid_y + \
        button_bounds_h / 2 + STEP_BUTTON_BOUNDING_BOX.top

    if selected:
        outline_color = GREEN
    else:
        outline_color = SURFACE_2

    if playing:
        led_color = RED
    else:
        led_color = SURFACE_2

    button = pygame.Rect(center_x, center_y, button_w, button_h)
    button.center = (center_x, center_y)
    pygame.draw.rect(screen, outline_color, button,
                     BUTTON_BOARDER_W, border_radius=BOARDER_RADIUS)
    y = button.bottom - BUTTON_BOARDER_W * 4
    pygame.draw.line(screen, led_color, (button.left + BUTTON_BOARDER_W *
                     3.5, y), (button.right - BUTTON_BOARDER_W * 3.5, y), BOARDER_RADIUS)

    # TODO: add a text display for the playing note


def draw_steps_buttons():
    for i in range(16):
        button_grid_x = i % 8
        button_grid_y = i // 8

        draw_a_button(i, button_grid_x, button_grid_y, False, False)
