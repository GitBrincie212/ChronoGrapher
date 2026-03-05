import { DocsLayout } from "fumadocs-ui/layouts/docs";
import ProgrammingLangToggle from "@/components/ui/toggle-language";
import { baseOptions } from "@/lib/layout.shared";
import { source } from "@/lib/source";

export default function Layout({ children }: LayoutProps<"/docs">) {
  return (
    <DocsLayout
      tree={source.getPageTree()}
      sidebar={{
        banner: <ProgrammingLangToggle />,
      }}
      {...baseOptions()}
    >
      {children}
    </DocsLayout>
  );
}
