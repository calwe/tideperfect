import { HashRouter, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import Home from './home/Home';
import Login from './login/Login';
import Album from './album/Album';
import Layout from './Layout';
import Playlist from './playlist/Playlist';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <main>
        <HashRouter>
          <Routes>
            <Route path="/login" element={<Login />} />
            <Route element={<Layout />}>
              <Route path="/" element={<Home />} />
              <Route path="/album/:albumId" element={<Album />} />
              <Route path="/playlist/:playlistId" element={<Playlist />} />
            </Route>
          </Routes>
        </HashRouter>
      </main>
    </QueryClientProvider>
  );
}

export default App;
