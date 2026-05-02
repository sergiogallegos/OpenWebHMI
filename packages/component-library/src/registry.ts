import type { ComponentDefinition } from "./types";
import { AlarmBanner } from "./components/AlarmBanner";
import { AlarmTable } from "./components/AlarmTable";
import { BarChart } from "./components/BarChart";
import { Button } from "./components/Button";
import { Card } from "./components/Card";
import { Container } from "./components/Container";
import { DataGrid } from "./components/DataGrid";
import { Divider } from "./components/Divider";
import { Dropdown } from "./components/Dropdown";
import { Gauge } from "./components/Gauge";
import { Image } from "./components/Image";
import { Indicator } from "./components/Indicator";
import { Label } from "./components/Label";
import { Modal } from "./components/Modal";
import { MultiState } from "./components/MultiState";
import { NumericInput } from "./components/NumericInput";
import { PieChart } from "./components/PieChart";
import { ProgressBar } from "./components/ProgressBar";
import { Slider } from "./components/Slider";
import { Spinner } from "./components/Spinner";
import { Stepper } from "./components/Stepper";
import { Tabs } from "./components/Tabs";
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
  [Tabs.kind]: Tabs,
  [Modal.kind]: Modal,
  [DataGrid.kind]: DataGrid,
  [BarChart.kind]: BarChart,
  [PieChart.kind]: PieChart,
  [Card.kind]: Card,
  [Spinner.kind]: Spinner,
  [Divider.kind]: Divider,
  [Stepper.kind]: Stepper,
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
  Tabs,
  Modal,
  DataGrid,
  BarChart,
  PieChart,
  Card,
  Spinner,
  Divider,
  Stepper,
] as const;
