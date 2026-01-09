import type { JSX } from "react";

type Color =
  | "info"
  | "warn"
  | "idea"
  | "error"
  | "success"
  | "neutral"
  | "primary"
  | "secondary";

const variants: Record<Color, string> = {
  info: "text-fd-info bg-fd-info/10",
  warn: "text-fd-warning bg-fd-warning/10",
  error: "text-fd-error bg-fd-error/10",
  idea: "text-fd-idea bg-fd-idea/10",
  success: "text-fd-success bg-fd-success/10",
  neutral: "text-fd-foreground bg-fd-foreground/10",
  primary: "text-fd-brand-primary bg-fd-brand-primary/10",
  secondary: "text-fd-brand-secondary bg-fd-brand-secondary/10",
};

export default function Highlight(props: {
  color: Color;
  children: JSX.Element | string;
  mono?: boolean;
  pulse?: boolean;
}) {
  return (
    <span className={"relative"}>
      <mark
        className={`${props.pulse ? "absolute" : ""} ${variants[props.color]} duration-200 px-1 font-semibold ${props.mono ? "font-mono" : ""}`}
      >
        {props.children}
      </mark>
      {props.pulse === true && (
        <mark
          className={`${variants[props.color]} animate-ping duration-200 bg-linear-to-br! px-1 font-semibold ${props.mono ? "font-mono" : ""}`}
        >
          {props.children}
        </mark>
      )}
    </span>
  );
}
