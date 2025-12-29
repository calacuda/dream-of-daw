import pygame
from .config import *


def mk_white_key(i):
    key = pygame.Rect(
        SIDE_BARS_W, SCREEN_CENTER[1],
        PIANO_WHITE_W, PIANO_H)
    return key


def mk_black_key(i):
    key = pygame.Rect(
        SIDE_BARS_W + PIANO_BLACK_W * i, SCREEN_CENTER[1],
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
    white_keys = [mk_white_key(i) for i in range(n_white_keys)]
    black_keys = [mk_black_key(i) for i in range(88 - n_white_keys)]
    white_i = 0
    black_i = 0

    for (i, midi_note) in [i for i in enumerate(range(24, 108)) if note_is_natural(i[1])]:
        # if note_is_natural(midi_note):
        color = TEXT
        offset = OCTAVE_W * (i // 12)
        key = white_keys[white_i]
        key.left += offset + (white_i % 7) * key.width
        key.width *= 0.9
        white_i += 1

        if (midi_note == midi_notes[step_i] and playing) or midi_note == midi_notes[cursor_position]:
            color = SURFACE_2

        pygame.draw.rect(
            screen, color, key, border_radius=BOARDER_RADIUS,
            border_top_left_radius=0 if i else BOARDER_RADIUS,
            border_top_right_radius=0 if white_i != n_white_keys else BOARDER_RADIUS
        )

    for (i, midi_note) in [i for i in enumerate(range(24, 108)) if not note_is_natural(i[1])]:
        # if not note_is_natural(midi_note):
        color = CRUST
        offset = OCTAVE_W * (i // 12)
        # print(f"n black key {len(black_keys)}, black_i {black_i}, i: {i}")
        key = black_keys[black_i]
        key.left = PIANO_BLACK_W * (i % 12) + offset + SIDE_BARS_W
        black_i += 1

        if (midi_note == midi_notes[step_i] and playing) or midi_note == midi_notes[cursor_position]:
            color = SURFACE_2

        pygame.draw.rect(
            screen, color, key, border_radius=BOARDER_RADIUS,
            border_top_left_radius=0, border_top_right_radius=0
        )
