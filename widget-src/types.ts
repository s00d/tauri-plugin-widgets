/** Dynamic widget IR node from `setWidgetConfig` JSON. */
export type ElNode = {
  type: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
};

export type ParentAxis = "horizontal" | "vertical" | undefined;

export type ColorValue =
  | string
  | {
      light?: string;
      dark?: string;
      colors?: string[];
      gradientType?: string;
      direction?: string;
    };

/** Matches `guest-js/generated/widget-types.ts` `WidgetConfig` plus desktop nonce. */
export type WidgetConfig = {
  version?: number;
  small?: ElNode;
  medium?: ElNode;
  large?: ElNode;
  __nonce?: number;
};

export type SkippedElement = { type: string; reason: string };
