from dataclasses import dataclass
from copy import deepcopy
from typing import Optional
import pygame
from .logger import log


A = 0
B = 1
X = 2
Y = 3
START = 9
SELECT = 8
LB = 4
RB = 5
LT = 6
RT = 7
HOME = 10
UP = (0, 1)
DOWN = (0, -1)
LEFT = (-1, 0)
RIGHT = (1, 0)


@dataclass
class ButtonData:
    pressed: bool
    just_pressed: bool
    just_released: bool
    pressed_time: int

    def step(self):
        if self.just_pressed:
            self.just_pressed = False

        if self.just_released:
            self.just_released = False

    def press(self):
        self.pressed = True
        self.just_pressed = True
        # TODO: set pressed time

    def release(self):
        self.pressed = False
        self.just_released = True


class Buttons:
    def __init__(self) -> None:
        self.pressed_now = {}
        self.last_pressed_now = {}

    def press(self, button):
        # if button in self.pressed_now:
        #     self.release(button)

        if button not in self.pressed_now:
            # log.debug(f"dpad pressed for {button}")
            # if self.pressed_now.get(button) is not None and type(button) is tuple:
            #     log.debug(f"dpad pressed for {
            #               self.last_pressed_now.get(button)}")
            #     return
            # else:
            self.pressed_now[button] = pygame.time.get_ticks()
            # log.debug(f"pressed: {button}")
        # else:

    def release(self, button):
        if button in self.pressed_now:
            self.pressed_now.pop(button)
            # log.debug(f"releasing {pygame.time.get_ticks() - value}")

    def purge_dpad(self):
        # log.debug(f"purging dpad: {self.pressed_now}")
        self.pressed_now = {
            but: time_stamp for (but, time_stamp) in self.pressed_now.items()
            if type(but) is not tuple}
        # log.debug(f"purging dpad: {self.pressed_now}")

    def is_pressed(self, button) -> Optional[int]:
        # return button in self.pressed_now.keys()
        time_stamp = self.pressed_now.get(button)

        if time_stamp is not None:
            time_stamp = pygame.time.get_ticks() - time_stamp

        return time_stamp

    def just_pressed(self, button) -> bool:
        return (self.is_pressed(button) is not None) and button not in self.last_pressed_now

    def just_released(self, button) -> Optional[int]:
        last = self.last_pressed_now.get(button)
        # log.debug(f"{button} => {last} and not {self.is_pressed(button)} = {
        #           last and not self.is_pressed(button)}")

        if last and not self.is_pressed(button):
            # print(f"keys = {self.last_pressed_now.keys()}")
            value = pygame.time.get_ticks() - last
            # log.debug(f"just_released {value}")

            return value
        else:
            return None

    def step(self):
        self.last_pressed_now = deepcopy(self.pressed_now)
