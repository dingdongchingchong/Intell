import { redirect } from "next/navigation";

/** Primary UI is the original Investigation Manager SPA. */
export default function HomePage() {
  redirect("/investigation.html");
}
