import pygame
from .config import *
from math import cos, sin, radians, sqrt


top = STEP_BUTTON_BOUNDING_BOX.top
left = STEP_BUTTON_BOUNDING_BOX.right
center_x = (left + SCREEN_WIDTH) * 0.5
multiplier = 2.5
bottom_half_top = ((STEP_BUTTON_BOUNDING_BOX.top + SCREEN_HEIGHT)
                   * 0.5)
play_top = (bottom_half_top + SCREEN_HEIGHT) * 0.5
p_center = ((center_x + left) * 0.5, (play_top + SCREEN_HEIGHT) * 0.5)
s_center = ((center_x + SCREEN_WIDTH) * 0.5, (play_top + SCREEN_HEIGHT) * 0.5)
# p_b_center = ((center_x + left) * 0.5, (play_top + SCREEN_HEIGHT) * 0.5)
distance = (center_x - left) * 0.125

p_b_center = (center_x + left) * 0.5
# p_b_center = (left + (center_x - left) * 0.485,
ratio = sqrt(distance**2 - (distance / 2)**2) / distance
p_b_center = (left + (center_x - left) * (ratio / 2 * 1.125),
              (play_top + SCREEN_HEIGHT) * 0.5)


def draw_bpm(font, font_2, bpm):
    text = "BPM:"
    text = font.render(text, True, TEXT)
    text_rect = text.get_rect(centerx=center_x, top=top)
    screen.blit(text, text_rect)

    text = str(bpm)
    center = (center_x, (text_rect.bottom + text_rect.bottom +
              text_rect.h * multiplier) * 0.5)
    button = pygame.Rect(center[0], center[1], (SCREEN_WIDTH - left) * 0.8,
                         text_rect.h * multiplier)
    button.center = center
    pygame.draw.rect(screen, SURFACE_2, button,
                     BUTTON_BOARDER_W, border_radius=BOARDER_RADIUS)

    text = str(bpm)
    text = font_2.render(text, True, TEXT)
    text_rect = text.get_rect(center=center)
    screen.blit(text, text_rect)


def draw_settings_button(font):
    center_y = (bottom_half_top + SCREEN_HEIGHT) * 0.5
    center = (center_x, (center_y + bottom_half_top) * 0.5)

    button = pygame.Rect(center[0], center[1], (SCREEN_WIDTH - left) * 0.8,
                         (SCREEN_HEIGHT - bottom_half_top) * 0.5 * 0.8)
    button.center = center
    pygame.draw.rect(screen, SURFACE_2, button,
                     BUTTON_BOARDER_W, border_radius=BOARDER_RADIUS)

    text = "Setting"
    text = font.render(text, True, TEXT)
    text_rect = text.get_rect(center=center)
    screen.blit(text, text_rect)


def mk_triangle(center, distance):
    def x(angle):
        # print(f"cos({angle}) = {cos(angle)}")
        return (distance * cos(radians(angle))) + center[0]

    def y(angle):
        # print(f"sin({angle}) = {sin(angle)}")
        return (distance * sin(radians(angle))) + center[1]

    def mk_point(angle):
        # print(f"(x, y) = {(x(angle), y(angle))}")
        return (x(angle), y(angle))

    p_1 = mk_point(0.0)
    p_2 = mk_point(120.0)
    p_3 = mk_point(240.0)

    return [p_1, p_2, p_3]


triangle_points = mk_triangle(p_b_center, distance)


def draw_play_button(playing, selected):
    # p_center = ((center_x + left) * 0.5, (play_top + SCREEN_HEIGHT) * 0.5)
    p_button = pygame.Rect(p_center[0], p_center[1], (center_x - left) * 0.75,
                           (SCREEN_HEIGHT - bottom_half_top) * 0.5 * 0.75)
    p_button.center = p_center
    color = SURFACE_2

    if selected:
        color = GREEN

    pygame.draw.rect(screen, color, p_button,
                     BUTTON_BOARDER_W, border_radius=BOARDER_RADIUS)
    color = TEXT

    if playing:
        color = GREEN

    pygame.draw.polygon(screen, color, triangle_points)

    # text = "Setting"
    # text = font.render(text, True, TEXT)
    # text_rect = text.get_rect(center=center)
    # screen.blit(text, text_rect)


def draw_stop_button(playing, selected):
    s_button = pygame.Rect(s_center[0], s_center[1], (center_x - left) * 0.75,
                           (SCREEN_HEIGHT - bottom_half_top) * 0.5 * 0.75)
    s_button.center = s_center
    color = SURFACE_2

    if selected:
        color = RED

    pygame.draw.rect(screen, color, s_button,
                     BUTTON_BOARDER_W, border_radius=BOARDER_RADIUS)
    color = TEXT

    if playing:
        color = RED

    s_button = pygame.Rect(
        s_center[0], s_center[1], s_button.h * 0.4, s_button.h * 0.4)
    s_button.center = s_center
    pygame.draw.rect(screen, color, s_button)


def draw_bottom_right_menu(font, font_2, playing, bpm):
    draw_bpm(font, font_2, bpm)
    draw_settings_button(font)
    draw_play_button(playing, False)
    draw_stop_button(playing, False)
