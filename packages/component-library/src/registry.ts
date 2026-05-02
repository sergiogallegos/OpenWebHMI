import type { ComponentDefinition } from "./types";
import { AlarmBanner } from "./components/AlarmBanner";
import { AlarmTable } from "./components/AlarmTable";
import { Button } from "./components/Button";
import { Container } from "./components/Container";
import { Dropdown } from "./components/Dropdown";
import { Gauge } from "./components/Gauge";
import { Image } from "./components/Image";
import { Indicator } from "./components/Indicator";
import { Label } from "./components/Label";
import { MultiState } from "./components/MultiState";
import { NumericInput } from "./components/NumericInput";
import { ProgressBar } from "./components/ProgressBar";
import { Slider } from "./components/Slider";
import { ToggleSwitch } from "./components/ToggleSwitch";
import { Trend } from "./components/Trend";
import { ValueDisplay } from "./components/ValueDisplay";

export const componentRegistry: Record<string, ComponentDefinition> = {
  [Label.kind]: Label,
  [ValueDisplay.kind]: ValueDisplay,
  [NumericInput.kind]: NumericInput,
  [Indicator.kind]: Indicator,
  [Image.kind]: Image,
  [Container.kind]: Container,
  [AlarmTable.kind]: AlarmTable,
  [Trend.kind]: Trend,
  [Gauge.kind]: Gauge,
  [ProgressBar.kind]: ProgressBar,
  [Slider.kind]: Slider,
  [Dropdown.kind]: Dropdown,
  [ToggleSwitch.kind]: ToggleSwitch,
  [Button.kind]: Button,
  [MultiState.kind]: MultiState,
  [AlarmBanner.kind]: AlarmBanner,
};

export const components = [
  Label,
  ValueDisplay,
  NumericInput,
  Indicator,
  Image,
  Container,
  AlarmTable,
  Trend,
  Gauge,
  ProgressBar,
  Slider,
  Dropdown,
  ToggleSwitch,
  Button,
  MultiState,
  AlarmBanner,
] as const;
