import { CodeBlock, Pre } from "fumadocs-ui/components/codeblock";
import {
  DocsBody,
  DocsDescription,
  DocsPage,
  DocsTitle,
} from "fumadocs-ui/layouts/docs/page";
import { createRelativeLink } from "fumadocs-ui/mdx";
import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { LLMCopyButton, ViewOptions } from "@/components/page-actions";
import Highlight from "@/components/ui/highlight";
import ThemeBasedImage from "@/components/ui/theme-based-image";
import {
  ProgrammingLanguage,
  RenderProgrammingLanguageBased,
} from "@/components/ui/toggle-language";
import { getPageImage, source } from "@/lib/source";
import { getMDXComponents } from "@/mdx-components";

export default async function Page(props: PageProps<"/docs/[[...slug]]">) {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) notFound();

  const MDX = page.data.body;

  // GitHub repository configuration
  const owner = "GitBrincie212";
  const repo = "ChronoGrapher";

  return (
    <DocsPage toc={page.data.toc} full={page.data.full}>
      <div className="flex flex-row gap-2 items-center -mb-4">
          <DocsTitle>{page.data.title}</DocsTitle>
          {page.data.optional && (
              <div className={"bg-linear-to-r from-fd-brand-primary to-fd-brand-secondary p-px rounded-[0.425rem]"}>
                  <div className={
                      "rounded-md px-2 font-semibold bg-fd-background select-none"
                  }>
                      <div className={"bg-linear-to-r from-fd-brand-primary to-fd-brand-secondary text-transparent bg-clip-text"}>OPTIONAL</div>
                  </div>
              </div>
          )}
      </div>
      <DocsDescription>{page.data.description}</DocsDescription>
      <DocsBody>
          <div className="flex flex-row gap-2 items-center -mt-10 border-b pb-4">
              <LLMCopyButton markdownUrl={`${page.url}.mdx`} />
              <ViewOptions
                  markdownUrl={`${page.url}.mdx`}
                  githubUrl={`https://github.com/${owner}/${repo}/blob/master/docs/content/docs/${page.path}`}
              />
          </div>

        <MDX
          components={getMDXComponents({
            // this allows you to link to other pages with relative file paths
            a: createRelativeLink(source, page),
            ThemeBasedImage: ThemeBasedImage,
            RenderProgrammingLanguageBased: RenderProgrammingLanguageBased,
            ProgrammingLanguage: ProgrammingLanguage,
            Highlight: Highlight,
            pre: ({ ref: _ref, ...props }) => (
              <CodeBlock {...props}>
                <Pre>{props.children}</Pre>
              </CodeBlock>
            ),
          })}
        />
        <div className="text-fd-muted-foreground opacity-50">
          Last Updated: {page.data.lastModified?.toDateString()}
        </div>
      </DocsBody>
    </DocsPage>
  );
}

export async function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata(
  props: PageProps<"/docs/[[...slug]]">,
): Promise<Metadata> {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) notFound();

  return {
    title: page.data.title,
    description: page.data.description,
    openGraph: {
      images: getPageImage(page).url,
    },
  };
}
