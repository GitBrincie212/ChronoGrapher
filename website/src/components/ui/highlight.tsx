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
  warn: "dark:text-fd-warning text-[color:color-mix(in_lch,var(--color-fd-warning)_60%,#c98112)] bg-fd-warning/10",
  error: "text-fd-error bg-fd-error/10",
  idea: "dark:text-fd-idea text-[color:color-mix(in_lch,var(--color-fd-idea)_50%,#9e4e00)] bg-fd-idea/25 dark:bg-fd-idea/10",
  success: "dark:text-fd-success text-[color:color-mix(in_lch,var(--color-fd-success)_50%,#02b84b)] bg-fd-success/25 dark:bg-fd-success/10",
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
        className={`${props.pulse ? "absolute" : ""} ${variants[props.color]} rounded-sm duration-200 px-1 font-semibold ${props.mono ? "font-mono" : ""}`}
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
