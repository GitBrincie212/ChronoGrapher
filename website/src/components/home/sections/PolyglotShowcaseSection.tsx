import SectionWire from "@/components/home/SectionWire";
import Highlight from "@/components/ui/highlight";

export default function PolyglotShowcaseSection() {
  return (
    <SectionWire
      leftRight={false}
      height={28}
      title={
        <>
          Learn Once, <u>Use In Any Programming Language</u>
        </>
      }
      description={
        <>
          <span className={"opacity-60"}>
            Designed to be used in any programming language and ecosystem,
            easily integrating with various established libraries / frameworks /
            platforms.
          </span>
          <span> </span>
          <Highlight color={"primary"}>
            No more polyglot fragmentation
          </Highlight>
        </>
      }
      icon={
        <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 512 512">
          <path d="M0 0h512v512H0z" fill="none" />
          <path fill="currentColor" d="m478.33 433.6l-90-218a22 22 0 0 0-40.67 0l-90 218a22 22 0 1 0 40.67 16.79L316.66 406h102.67l18.33 44.39A22 22 0 0 0 458 464a22 22 0 0 0 20.32-30.4ZM334.83 362L368 281.65L401.17 362Zm-66.99-19.08a22 22 0 0 0-4.89-30.7c-.2-.15-15-11.13-36.49-34.73c39.65-53.68 62.11-114.75 71.27-143.49H330a22 22 0 0 0 0-44H214V70a22 22 0 0 0-44 0v20H54a22 22 0 0 0 0 44h197.25c-9.52 26.95-27.05 69.5-53.79 108.36c-31.41-41.68-43.08-68.65-43.17-68.87a22 22 0 0 0-40.58 17c.58 1.38 14.55 34.23 52.86 83.93c.92 1.19 1.83 2.35 2.74 3.51c-39.24 44.35-77.74 71.86-93.85 80.74a22 22 0 1 0 21.07 38.63c2.16-1.18 48.6-26.89 101.63-85.59c22.52 24.08 38 35.44 38.93 36.1a22 22 0 0 0 30.75-4.9Z" />
        </svg>
      }
      iconColorClass={"bg-fd-brand-primary/5 dark:bg-fd-brand-primary/20 border-fd-brand-primary text-fd-brand-primary"}
    >
      <div></div>
    </SectionWire>
  );
}
