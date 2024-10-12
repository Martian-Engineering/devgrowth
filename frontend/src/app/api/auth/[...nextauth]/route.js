import NextAuth from "next-auth";
import GithubProvider from "next-auth/providers/github";
import { logError } from "@/lib/logger";

export const authOptions = {
  session: {
    strategy: "jwt",
  },
  providers: [
    GithubProvider({
      clientId: process.env.GITHUB_CLIENT_ID,
      clientSecret: process.env.GITHUB_CLIENT_SECRET,
      scope: "read:user user:email",
    }),
  ],
  callbacks: {
    async jwt({ token, account, profile }) {
      if (account && profile) {
        token.accessToken = account.access_token;
        token.id = profile.id;
      }
      return token;
    },
    async session({ session, token }) {
      session.user.id = token.id;
      session.isAuthenticated = !!token.accessToken; // boolean flag
      return session;
    },
  },
  secret: process.env.NEXTAUTH_SECRET,
  debug: process.env.NODE_ENV === "development",
  logger: {
    error: (code, metadata) => {
      logError(`NextAuth error: ${code}`, metadata);
    },
    warn: (code) => {
      console.warn(`NextAuth warning: ${code}`);
    },
    debug: (code, metadata) => {
      console.log(`NextAuth debug: ${code}`, metadata);
    },
  },
};

const handler = NextAuth(authOptions);
export { handler as GET, handler as POST };
