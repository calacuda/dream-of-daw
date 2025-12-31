import pygame
from .config import *
from do_daw import N_CHANNELS


def draw_channel_button(i, font, color, text):
    mid_y = STEP_BUTTON_BOUNDING_BOX.top + \
        ((STEP_BUTTON_BOUNDING_BOX.height / (N_CHANNELS + 1)) * (i + 1))
    center = (CHANNEL_MID_X, mid_y)
    button_w = STEP_BUTTON_BOUNDING_BOX.left - SIDE_BARS_W
    button = pygame.Rect(center[0], center[1], button_w * 0.75,
                         (STEP_BUTTON_BOUNDING_BOX.height / (N_CHANNELS + 1)) * 0.8)
    button.center = center
    pygame.draw.rect(screen, color, button,
                     BUTTON_BOARDER_W, border_radius=BOARDER_RADIUS)

    text = text[:12] if text is not None else "?"
    text = font.render(text, True, TEXT)
    text_rect = text.get_rect(center=center)
    screen.blit(text, text_rect)


def draw_channel_switcher(font, channel_i, selected, plugins):
    for i in range(N_CHANNELS):
        color = SURFACE_2

        if channel_i == i:
            color = SURFACE_0

        if selected == i:
            color = GREEN

        draw_channel_button(i, font, color, plugins[i])
