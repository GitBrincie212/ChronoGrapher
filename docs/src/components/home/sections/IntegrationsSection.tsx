/** biome-ignore-all lint/a11y/noSvgWithoutTitle: Shut up */

import ProgrammingLanguagesShowcase from "@/components/home/ProgrammingLanguages";
import WebFrameworkBrowserWindow from "@/components/home/WebFrameworkWindow";

function TextDescription(props: { children: string }) {
  return (
    <div
      className={
        "absolute w-full bottom-0 right-0 px-2 pt-6 z-10 text-xs font-[VioletSans] text-fd-card-foreground/50 " +
        "bg-linear-to-b from-transparent to-black/5 to-50% pb-2"
      }
    >
      {props.children}
    </div>
  );
}

export default function IntegrationsSection() {
  return (
    <div className={"w-screen min-h-3xl mt-18 mb-12"}>
      <h1 className={"text-4xl font-bold text-center w-full"}>
        Integratable With Any Ecosystem
      </h1>
      <div
        className={
          "text-xl opacity-50 w-full text-center mb-8 font-[VioletSans] "
        }
      >
        ChronoGrapher can work in various environments, programming languages
        and frameworks
      </div>
      <div className={"flex h-140 w-screen justify-center px-18"}>
        <div
          className={
            "grid grid-cols-5 grid-rows-6 gap-4 w-full *:bg-fd-background-100p *:rounded-lg *:relative " +
            "*:ring-1 *:ring-offset-4 *:ring-fd-background-100p *:ring-offset-fd-background *:p-3"
          }
        >
          <div className="col-span-1 row-span-3 flex overflow-hidden relative">
            <ProgrammingLanguagesShowcase />
            <TextDescription>
              Made to be polyglot, works in lots of programming languages with
              the same unified API but more native.
            </TextDescription>
            <div
              className="absolute inset-0 z-5 opacity-25 dark:opacity-10 mix-blend-soft-light"
              style={{
                backgroundImage: `
        linear-gradient(to right, var(--color-fd-foreground) 1px, transparent 1px),
        linear-gradient(to bottom, var(--color-fd-foreground) 1px, transparent 1px)
      `,
                backgroundSize: "32px 32px, 32px 32px, 100% 100%",
              }}
            />
          </div>
          <div className="col-span-5 row-span-3 overflow-hidden ">
            <div className={"relative h-full"}>
              <WebFrameworkBrowserWindow
                position={[0, 0]}
                titles={["Django", "FastAPI"]}
                icons={[
                  <svg
                    key={"django-icon"}
                    xmlns="http://www.w3.org/2000/svg"
                    width="1em"
                    height="1em"
                    viewBox="0 0 24 24"
                  >
                    <path
                      fill="currentColor"
                      fillRule="evenodd"
                      d="M15.172 0h-4.176v5.932a7.2 7.2 0 0 0-1.816-.2C4.816 5.731 2 8.305 2 12.273c0 4.118 2.655 6.263 7.755 6.268c1.703 0 3.278-.15 5.417-.53zM9.734 8.977c.516 0 .92.05 1.408.2v6.248c-.596.075-.972.1-1.434.1c-2.14 0-3.305-1.142-3.305-3.21c0-2.125 1.22-3.338 3.331-3.338"
                      clipRule="evenodd"
                    />
                    <path
                      fill="currentColor"
                      d="M22 15.233V6.215h-4.17v7.675c0 3.387-.188 4.674-.785 5.786c-.57 1.087-1.462 1.8-3.305 2.606L17.615 24c1.843-.862 2.735-1.643 3.412-2.88c.726-1.288.973-2.782.973-5.887M21.585 0h-4.176v3.993h4.176z"
                    />
                  </svg>,
                  <svg
                    key={"fastapi-icon"}
                    xmlns="http://www.w3.org/2000/svg"
                    width="1em"
                    height="1em"
                    viewBox="0 0 24 24"
                  >
                    <path
                      fill="currentColor"
                      d="M12 .039c-6.627 0-12 5.354-12 11.96c-.001 6.606 5.372 11.963 12 11.962S24.001 18.605 24 12S18.627.039 12 .039m-.829 5.415h7.55l-7.58 5.329h5.182L5.28 18.543l5.891-13.088"
                    />
                  </svg>,
                ]}
                activeColors={[
                  ["rgb(102, 180, 142)", "rgb(24, 31, 27)"],
                  ["rgb(102,180,162)", "rgb(24,31,30)"],
                ]}
              />
              <WebFrameworkBrowserWindow
                position={[14, 0]}
                titles={["Laravel", "Symfony"]}
                icons={[
                  <svg
                    key={"laravel-icon"}
                    xmlns="http://www.w3.org/2000/svg"
                    width="1em"
                    height="1em"
                    viewBox="0 0 512 512"
                  >
                    <path
                      fill="currentColor"
                      d="M107.2 0c2.5 0 4.7.8 6.7 2l94.3 54.1c2.7 1.5 4.5 3.5 5.4 5.9c.9 2.2.9 4.3.9 5.6V261l69.2-39.7V121c0-2.6.6-5 2.2-7.2c1.5-2.1 3.5-3.6 5.7-4.8l94-54c1.6-.9 3.4-1.6 5.5-1.6s4 .7 5.6 1.6l95.8 55.1c2.3 1.3 3.9 3 4.9 5.3c.9 2.1.9 4.2.9 5.8v107.2c0 2-.2 4.3-1.4 6.4c-1.2 2.2-3 3.7-5.1 4.9l-.1.1l-88 50.5v100c0 2.3-.3 4.8-1.6 7s-3.3 3.7-5.3 4.9h-.1L208.7 510c-2.2 1.2-4.5 2-7.1 2s-4.9-.9-7.1-2l-.1-.1L7.1 402l-.5-.3c-1.1-.7-2.6-1.7-3.8-2.9c-1.9-1.9-2.8-4.2-2.8-7.2V65.9c0-4.8 3-7.9 5.5-9.3l95-54.6c2-1.2 4.3-2 6.8-2zM38.1 67.1l69 39.9l69.2-39.9l-69.2-39.7zm353 93.2l69-39.7l-69-39.7l-69.1 39.7zM189.2 89L120 128.8v186.4l69.2-39.9V88.9zm-94.7 39.9L25.2 89.1v294.2l164 94.2v-79.4l-87.3-49.3l-.2-.1c-1.3-.8-3.2-1.9-4.6-3.7c-1.7-2.1-2.5-4.7-2.5-7.7V128.8zm214.7 92.4l69.3 39.6v-78.5l-69.3-39.9zm94.5 39.6l69.3-39.7v-78.8l-69.3 39.9v78.5zM201.6 376.1l163.8-93.2l-69-39.9L133 337.1l68.6 38.9zm12.9 101.5l164-94.2v-78.8l-164 93.6z"
                    />
                  </svg>,
                  <svg
                    key={"symfony-icon"}
                    xmlns="http://www.w3.org/2000/svg"
                    width="1em"
                    height="1em"
                    viewBox="0 0 24 24"
                  >
                    <path
                      fill="currentColor"
                      d="M24 12c0 6.628-5.372 12-12 12S0 18.628 0 12S5.372 0 12 0s12 5.372 12 12m-6.753-7.561c-1.22.042-2.283.715-3.075 1.644c-.878 1.02-1.461 2.229-1.881 3.461c-.753-.614-1.332-1.414-2.539-1.761a3.1 3.1 0 0 0-2.813.514c-.41.319-.71.757-.861 1.254c-.36 1.176.381 2.225.719 2.6l.737.79c.15.154.519.56.339 1.138c-.193.631-.951 1.037-1.732.799c-.348-.106-.848-.366-.734-.73c.045-.15.152-.263.21-.391c.052-.11.077-.194.095-.242c.141-.465-.053-1.07-.551-1.223c-.465-.143-.939-.03-1.125.566c-.209.68.117 1.913 1.86 2.449c2.04.628 3.765-.484 4.009-1.932c.153-.907-.255-1.582-1.006-2.447l-.612-.677c-.371-.37-.497-1.002-.114-1.485c.324-.409.785-.584 1.539-.379c1.103.3 1.594 1.063 2.412 1.68c-.338 1.11-.56 2.223-.759 3.222l-.123.746c-.585 3.07-1.033 4.757-2.194 5.726c-.234.166-.57.416-1.073.434c-.266.005-.352-.176-.355-.257c-.006-.184.15-.271.255-.353c.154-.083.39-.224.372-.674c-.016-.532-.456-.994-1.094-.973c-.477.017-1.203.465-1.176 1.286c.028.85.819 1.485 2.012 1.444c.638-.021 2.062-.281 3.464-1.949c1.633-1.911 2.09-4.101 2.434-5.706l.383-2.116q.318.037.69.048c2.032.044 3.049-1.01 3.064-1.776c.01-.464-.304-.921-.744-.91a.85.85 0 0 0-.806.654c-.094.428.646.813.068 1.189c-.41.266-1.146.452-2.184.3l.188-1.042c.386-1.976.859-4.407 2.661-4.467c.132-.007.612.006.623.323c.003.105-.022.134-.147.375a.86.86 0 0 0-.168.537c.017.504.4.836.957.816c.743-.023.955-.748.945-1.119c-.032-.874-.952-1.424-2.17-1.386"
                    />
                  </svg>,
                ]}
                activeColors={[
                  ["rgb(199, 72, 66)", "rgb(27,20,20)"],
                  ["rgb(200,200,200)", "rgb(29,29,29)"],
                ]}
              />
              <WebFrameworkBrowserWindow
                position={[28, 0]}
                titles={["Spring Boot"]}
                icons={[
                  <svg
                    key={"springboot-icon"}
                    xmlns="http://www.w3.org/2000/svg"
                    width="1em"
                    height="1em"
                    viewBox="0 0 24 24"
                  >
                    <path
                      fill="currentColor"
                      d="M21.854 1.416a10.5 10.5 0 0 1-1.284 2.247A11.967 11.967 0 1 0 3.852 20.776l.444.395a11.954 11.954 0 0 0 19.632-8.297c.346-3.013-.568-6.865-2.074-11.458M5.58 20.875a1.017 1.017 0 1 1-.149-1.433a1.04 1.04 0 0 1 .149 1.432m16.199-3.581c-2.939 3.926-9.26 2.593-13.286 2.79c0 0-.716.05-1.432.148c0 0 .272-.123.618-.247c2.84-.987 4.173-1.185 5.901-2.074c3.235-1.654 6.47-5.284 7.112-9.038c-1.235 3.606-4.988 6.717-8.396 7.976c-2.346.865-6.568 1.704-6.568 1.704l-.173-.099c-2.865-1.407-2.963-7.63 2.272-9.63c2.296-.89 4.47-.395 6.963-.988c2.643-.617 5.705-2.593 6.94-5.186c1.382 4.174 3.061 10.643.049 14.644"
                    />
                  </svg>,
                ]}
                activeColors={[["rgb(135, 185, 85)", "rgb(31,36,27)"]]}
              />
              <WebFrameworkBrowserWindow
                position={[36, 0]}
                titles={["Rails", "Sinatra"]}
                icons={[
                  <svg
                    key={"ruby-rails-icon"}
                    xmlns="http://www.w3.org/2000/svg"
                    width="1em"
                    height="1em"
                    viewBox="0 0 128 128"
                  >
                    <path
                      fill="currentColor"
                      fillRule="evenodd"
                      d="M109.682 14.737c-12.206-6.023-24.708-6.636-37.508-2.111c-11.779 4.164-21.175 11.615-28.16 21.763C32.195 51.561 23.61 70.298 18.799 90.652c-2.464 10.417-4.06 21.466-3.631 32.224c.035.873.165 1.124.251 3.124h60.366c-.173-2-.287-1.416-.437-1.797a154 154 0 0 1-7.428-25.198c-2.498-12.251-3.806-24.729-1.226-37.093c3.611-17.313 13.48-29.805 30.117-36.283c9.424-3.667 18.369-2.624 26.214 4.262c.072.063.22.025.412.056l2.565-3.883c-4.94-4.703-10.368-8.389-16.32-11.327M3.336 94.394c-.46 3.923-.89 7.596-1.34 11.451l11.132 1.336l2.039-11.893zm21.85-34.186l-10.471-4.097l-3.384 9.607l10.671 3.42c1.08-3.031 2.096-5.882 3.184-8.93m49.419 53.659c3.575.266 7.157.449 11.103.679c-1.433-2.979-2.706-5.673-4.039-8.335c-.146-.289-.639-.568-.974-.573c-3.033-.044-6.068-.025-9.291-.025c.726 2.628 1.357 5.053 2.096 7.443c.111.361.707.782 1.105.811M42.933 31.103l-7.955-5.268l-6.359 7.105l8.178 5.496zm25.334 53.369c-.013.321.276.832.558.959c2.865 1.288 5.76 2.515 8.912 3.873c-.131-2.492-.219-4.575-.368-6.654c-.027-.374-.203-.912-.48-1.066c-2.631-1.456-5.299-2.847-8.216-4.395c-.159 2.665-.321 4.972-.406 7.283M65.91 12.3l-5.446-6.181l-7.499 3.898l5.455 6.644zm3.415 49.176c-.163.374.052 1.167.373 1.456c2.175 1.962 4.424 3.84 6.926 5.981c.573-2.4 1.113-4.539 1.571-6.693c.081-.383-.032-1.016-.298-1.23c-1.946-1.569-3.955-3.063-6.037-4.651c-.915 1.815-1.802 3.443-2.535 5.137m12.45-52.424c2.78.075 5.563.042 8.499.042c-.293-2.044-.433-3.593-.782-5.092c-.104-.446-.775-1.04-1.228-1.078c-2.787-.226-5.585-.313-8.651-.459c.409 2.063.721 3.881 1.162 5.668c.093.379.647.909 1 .919m3.385 35.675c.142-.266.178-.749.029-.981c-1.366-2.137-2.785-4.241-4.254-6.455l-4.76 4.372l6.582 7.294c.884-1.539 1.675-2.868 2.403-4.23M90.295 30.2l2.843 5.281c4.449-2.438 4.875-3.32 3.3-6.834zm21.287-16.273c1.851 1.142 3.806 2.115 5.792 3.185l1.33-2.07c-2.422-1.771-4.76-3.484-7.413-5.426c-.104 1.104-.259 1.875-.219 2.637c.032.581.129 1.44.51 1.674M109 30.646c2 .217 5 .424 7 .643v-2.718c-2-.438-5-.872-7-1.323z"
                      clipRule="evenodd"
                    />
                  </svg>,
                  <svg
                    key={"sinitra-icon"}
                    xmlns="http://www.w3.org/2000/svg"
                    width="1em"
                    height="1em"
                    viewBox="0 0 24 24"
                  >
                    <path
                      fill="currentColor"
                      d="M5.224 10.525c-1.215.4-3.32 1.384-3.943 1.934q-.103.09-.207.194c-.49.43-.76.851-.89 1.285A4.4 4.4 0 0 0 0 15.208c-.005.842.247 1.369.247 1.369c.714 1.428 2.416 2.4 4.21 2.963c1.777.593 5.622.289 7.521.046c5.091-.684 8.389-1.657 10.319-3.237C23.924 15.042 24 13.644 24 13.127a1.5 1.5 0 0 0-.02-.219c-.032-.553-.258-1.62-1.49-2.38a3 3 0 0 0-.33-.197q-.187-.098-.375-.186l-.035-.027l-.191-.078a12 12 0 0 0-.629-.264c-.515-.225-.971-.387-1.372-.477a70 70 0 0 1-.041-1.703c0-.7-.244-1.08-.441-1.277c-.12-.119-.557-.265-.997-.4a19 19 0 0 0-.93-.287l-.079-.027v.005l-.417-.12h-.001l-.448-.128l-.094-.015l-.033-.01l-.07-.02l-.028-.008l-.641-.19l-.091-.012v-.003l-.213-.057v-.004l-.32-.09v-.001a18 18 0 0 0-.669-.167a70 70 0 0 0-2.689-.502c-.182-.046-1.367-.152-1.367-.152a6 6 0 0 0-1.106.023a4 4 0 0 0-.779.19c-.113.024-.245.103-.383.216a1.4 1.4 0 0 0-.177.146l-.002.002l-.125.12l-.007.008c-.217.217-.36.412-.454.584c-.174.249-.304.479-.341.61c-.032.119-.129.578-.228 1.044c-.091.432-.184.871-.228 1.054c-.048.2-.334.906-.601 1.567c-.124.304-.243.598-.334.83m14.384.642c0-.06.076-.06.076.015c0 0 0 .016-.003.036l.003.025c0 .03 0 .456-.03.957q-.013.207-.031.426c-.011.144-.023.289-.03.425c-.007.225-.008.431-.007.59c.007.154.007.246.007.246v.106c0 .259-.152.593-.396.745l-.04.026h-.001l-.021.013a1.8 1.8 0 0 1-.409.23c-.22.106-.516.223-.942.339c-.836.243-1.459.35-1.869.395c-1.003.122-2.188.182-3.601.182c-.29 0-1.414 0-1.687-.015c-3.739-.106-5.988-1.23-5.988-2.036v-.106s.32-2.478.32-2.63s.09-.273.09-.182v.06l.002.093q.003.015.008.025t.006.02c.32 1.018 3.45 1.717 7.279 1.717h.638l.205-.003h.001c1.15-.012 3.818-.042 5.842-.954c.35-.228.578-.456.578-.745"
                    />
                  </svg>,
                ]}
                activeColors={[
                  ["rgb(192, 40, 28)", "rgb(27,19,18)"],
                  ["rgb(185,172,172)", "rgb(21,18,18)"],
                ]}
              />
            </div>
            <TextDescription>
              Interfaces well with many web frameworks and their own
              ecosystems such as Django for Python, Ruby On Rails for Ruby and
              even Laravel for PHP.
            </TextDescription>
            <div
              className="absolute inset-0 z-0 pointer-events-none opacity-25 dark:opacity-10 mix-blend-soft-light"
              style={{
                backgroundImage: `
                        repeating-linear-gradient(45deg, 
                          var(--color-fd-foreground) 0px, 
                          var(--color-fd-foreground) 2px, 
                          transparent 2px, 
                          transparent 25px
                        )
                      `,
              }}
            />
          </div>
          <div className="col-span-3 row-span-2">
            <TextDescription>
              Deploy ChronoGrapher to your favourite platform / cloud service,
              including but not limited to Google Clouds, Azure Cloud and AWS
              and many others.
            </TextDescription>
            <div
              className="absolute inset-0 z-0 mix-blend-soft-light opacity-80"
              style={{
                backgroundImage: `
       radial-gradient(circle at 25% 25%, var(--color-fd-foreground) 0.5px, transparent 1px),
       radial-gradient(circle at 75% 75%, var(--color-fd-foreground) 0.5px, transparent 1px)
     `,
                backgroundSize: "10px 10px",
                imageRendering: "pixelated",
                WebkitMask:
                  "linear-gradient(to bottom, black 0, transparent 70%)",
                mask: "linear-gradient(to bottom, black 0, transparent 70%)",
              }}
            />
          </div>
          <div className="col-span-3 row-span-2 row-start-4 col-start-4">
            <div
              className={
                "flex z-10 justify-center text-7xl gap-4 w-full mt-12 mask-linear-[90deg,#FFFFFF2F_0%,white_50%,#FFFFFF2F_100%] to-transparent"
              }
            ></div>
            <TextDescription>
              Works seamlessly with distributed systems related tools such as
              Kubernetes, Prometheus, etcd and many others.
            </TextDescription>
            <div className={"contents"}>
              <div
                className="absolute inset-0 z-10 opacity-30 mix-blend-soft-light"
                style={{
                  backgroundImage: `
        linear-gradient(90deg, var(--color-fd-foreground) 1px, transparent 0),
        linear-gradient(180deg, var(--color-fd-foreground) 1px, transparent 0)
      `,
                  backgroundSize: "24px 24px, 24px 24px, 24px 24px",
                  WebkitMask:
                    "radial-gradient(circle at var(--x, 50%) var(--y, 50%), black 0, transparent 70%)",
                  mask: "radial-gradient(circle at var(--x, 50%) var(--y, 50%), black 0, transparent 70%)",
                }}
              />
              <div
                className="absolute inset-0 z-5 opacity-20 mix-blend-soft-light"
                style={{
                  backgroundImage: `
        repeating-linear-gradient(45deg, var(--color-fd-foreground) 0 2px, transparent 2px 6px)
      `,
                  backgroundSize: "24px 24px, 24px 24px, 24px 24px",
                  WebkitMask:
                    "radial-gradient(circle at var(--x, 50%) var(--y, 50%), black 0, transparent 70%)",
                  mask: "radial-gradient(circle at var(--x, 50%) var(--y, 50%), black 0, transparent 70%)",
                }}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
