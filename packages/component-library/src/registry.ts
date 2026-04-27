import type { ComponentDefinition } from "./types";
import { Container } from "./components/Container";
import { Image } from "./components/Image";
import { Indicator } from "./components/Indicator";
import { Label } from "./components/Label";
import { NumericInput } from "./components/NumericInput";
import { ValueDisplay } from "./components/ValueDisplay";

export const componentRegistry: Record<string, ComponentDefinition> = {
  [Label.kind]: Label,
  [ValueDisplay.kind]: ValueDisplay,
  [NumericInput.kind]: NumericInput,
  [Indicator.kind]: Indicator,
  [Image.kind]: Image,
  [Container.kind]: Container,
};

export const components = [
  Label,
  ValueDisplay,
  NumericInput,
  Indicator,
  Image,
  Container,
] as const;
