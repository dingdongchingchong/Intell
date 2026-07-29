export { default } from "next-auth/middleware";

export const config = {
  matcher: [
    "/dashboard/:path*",
    "/cases/:path*",
    "/kanban/:path*",
    "/users/:path*",
    "/settings/:path*",
  ],
};
