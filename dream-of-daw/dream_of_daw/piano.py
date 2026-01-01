import pygame
from .config import *


def mk_white_key():
    key = pygame.Rect(
        SIDE_BARS_W, SCREEN_CENTER[1],
        PIANO_WHITE_W, PIANO_H)
    return key


def mk_black_key():
    key = pygame.Rect(
        SIDE_BARS_W, SCREEN_CENTER[1],
        PIANO_BLACK_W, PIANO_BLACK_H)
    return key


def note_is_natural(midi_note):
    note = midi_note % 12
    return note in [0, 2, 4, 5, 7, 9, 11]


def draw_piano(playing, step_i, midi_notes, cursor_position):
    """
    draw_piano fucntion. takes 3 args:
        - when the stepper is playing,
        - the playing step index,
        - a list of the midi notes for the current channel
        - the step selected by the cursor
    """
    white_keys = [mk_white_key() for _ in range(n_white_keys)]
    black_keys = [mk_black_key() for _ in range(5 * N_OCTAVES)]

    for (white_i, (i, midi_note)) in enumerate([i for i in enumerate(range(24, 108)) if note_is_natural(i[1])]):
        color = TEXT
        offset = OCTAVE_W * (i // 12)
        key = white_keys[white_i]
        key.left += offset + (white_i % 7) * key.width
        key.width *= 0.9

        if (midi_note == midi_notes[step_i] and playing) or (midi_note == midi_notes[cursor_position] and not playing):
            color = SURFACE_2

        pygame.draw.rect(
            screen, color, key, border_radius=BOARDER_RADIUS,
            border_top_left_radius=0 if i else BOARDER_RADIUS,
            border_top_right_radius=0 if white_i != n_white_keys else BOARDER_RADIUS
        )

    for (black_i, (i, midi_note)) in enumerate([i for i in enumerate(range(24, 108)) if not note_is_natural(i[1])]):
        color = CRUST
        offset = OCTAVE_W * (i // 12)
        key = black_keys[black_i]
        key.left = PIANO_BLACK_W * (i % 12) + offset + SIDE_BARS_W

        if (midi_note == midi_notes[step_i] and playing) or (midi_note == midi_notes[cursor_position] and not playing):
            color = SURFACE_2

        pygame.draw.rect(
            screen, color, key, border_radius=BOARDER_RADIUS,
            border_top_left_radius=0, border_top_right_radius=0
        )
