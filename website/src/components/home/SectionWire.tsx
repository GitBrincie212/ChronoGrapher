import type React from "react";

export default function SectionWire(props: {
  children: React.ReactElement;
  height: number;
  leftRight: boolean;
  title: React.ReactElement;
  description: React.ReactElement;
  icon: React.ReactElement;
  iconColorClass: string
}) {
  return (
    <div className={"w-full relative h-max"} style={{ height: `${props.height}rem` }}>
      <div className={`contents *:z-5 ${props.leftRight ? "*:border-l *:left-0" : "*:border-r *:right-0"} *:border-fd-muted-foreground/20 *:absolute *:h-full`}>
          <div className={props.leftRight ? "ml-[5.1rem]" : "mr-[7.85rem]"} />
          <div className={props.leftRight ? "ml-[4.85rem]" : "mr-[8.15rem]"}/>
      </div>
      <div className={`${props.leftRight ? "ml-[4.9rem]" : "text-end mr-36"} px-4 py-8 h-full relative`}>
        <div className={`flex flex-col w-full h-full relative ${props.leftRight ? "ml-2" : "items-end mr-32"}`}>
          <div className={`relative flex items-center mb-1 gap-2 ${props.leftRight ? "" : "flex-row-reverse"}`}>
              <div className={`size-8 border rounded-lg flex justify-center items-center text-lg ${props.iconColorClass}`}>
                  <div className={
                      `absolute text-lg flex justify-center items-center blur-xs opacity-80 dark:opacity-100 
                      animate-pulse duration-300 z-10 ${props.iconColorClass} bg-transparent!`
                  }>
                      {props.icon}
                  </div>
                  {props.icon}
              </div>
              <h1 className={"font-bold text-md md:text-lg lg:text-xl"}>{props.title}</h1>
          </div>
          <div className={`w-[70ch] mb-10 text-xs md:text-sm lg:text-md ${props.leftRight ? "" : "text-end"}`}>
            {props.description}
          </div>
          {props.children}
        </div>
      </div>
      <div
          className={
              "contents *:border-b *:border-fd-muted-foreground/20 *:absolute *:w-full *:bottom-0"
          }
      >
          <div className={"mb-4"} />
          <div className={"mb-3"}/>
      </div>
      <div
        className={
          "ml-16 absolute bottom-0 z-10 size-8 bg-fd-background border border-fd-muted-foreground/20 rounded-lg"
        }
      />
      <div
        className={
          "mr-28 absolute bottom-0 right-0 z-10 size-8 bg-fd-background border border-fd-muted-foreground/20 rounded-lg"
        }
      />
    </div>
  );
}
