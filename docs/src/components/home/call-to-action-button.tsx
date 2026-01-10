const variants = {
  primary:
    "bg-fd-brand-primary/10 border-fd-brand-primary/50 hover:bg-fd-brand-primary hover:ring-fd-brand-primary/75",
  secondary:
    "bg-fd-brand-secondary/10 border-fd-brand-secondary/50 hover:bg-fd-brand-secondary hover:ring-fd-brand-secondary/75",
} as const;

export default function CallToActionButton(props: {
  title: string;
  variant: keyof typeof variants;
  href: string;
}) {
  return (
    <button
      type={"button"}
      onClick={() => {
        location.href = props.href;
      }}
      className={
        "relative h-12 overflow-hidden rounded font-semibold px-5 py-2.5 text-white transition cursor-pointer " +
        `border duration-150 hover:ring-2 hover:ring-offset-4 hover:ring-offset-fd-background ${variants[props.variant]}`
      }
    >
      <span className="relative">{props.title}</span>
    </button>
  );
}
