import ReactDOM from "react-dom/client";
import { createHashRouter, RouterProvider } from "react-router-dom";
import App from "./App";
import CommitDetailPage from "./components/CommitDetailPage";
import "./App.css";

const router = createHashRouter([
  {
    path: "/",
    element: <App />,
  },
  {
    path: "/commit/:repositoryId/:commitId",
    element: <CommitDetailPage />,
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <RouterProvider router={router} />
);