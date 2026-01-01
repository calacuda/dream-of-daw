import pygame
from .config import *
import string
from do_daw import N_SECTIONS
from do_daw import UiSector


center_x = SIDE_BARS_W / 2
section_h = (SCREEN_HEIGHT - HINT_BAR_H) / N_SECTIONS
discount = 0.8


def draw_section(font, display_name, i, color, text_color):
    center_y = HINT_BAR_H + (section_h * 0.5) + (section_h * i)
    center = (center_x, center_y)
    button = pygame.Rect(center[0], center[1], SIDE_BARS_W * discount,
                         section_h * discount)
    button.center = center
    pygame.draw.rect(screen, color, button,
                     BUTTON_BOARDER_W, border_radius=BOARDER_RADIUS)

    text = display_name
    text = font.render(text, True, text_color)
    text_rect = text.get_rect(center=center)
    screen.blit(text, text_rect)


def draw_sections(font, section_i):
    center_y = HINT_BAR_H * 0.5
    center = (center_x, center_y)
    text = "Section"
    text = font.render(text, True, TEXT)
    text_rect = text.get_rect(center=center)
    screen.blit(text, text_rect)
    in_sector = cursor.sector == UiSector.Sections

    for (i, display_name) in enumerate(string.ascii_uppercase[:N_SECTIONS]):
        color = SURFACE_0
        text_color = SURFACE_0

        if section_i == i:
            color = SURFACE_2

        if in_sector and cursor.index == i:
            color = GREEN
            text_color = TEXT

        if section_i == i:
            # text_color = SAPHIRE
            text_color = GREEN

        draw_section(font, display_name, i, color, text_color)
