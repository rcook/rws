from collections import defaultdict

def topo_sort_helper(graph, v, visited, stack):
    visited[v] = True
    for i in graph[v]:
        if not visited[i]:
            topo_sort_helper(graph, i, visited, stack)
    stack.insert(0, v)

class Graph(object):
    def __init__(self):
        self._count = 0
        self._graph = defaultdict(list)

    def add_edge(self, u, v):
        if u < 0:
            raise ValueError("Invalid vertex {}".format(u))
        if v < 0:
            raise ValueError("Invalid vertex {}".format(v))
        self._count = max(self._count, u + 1, v + 1)
        self._graph[u].append(v)

    def topo_sort(self):
        visited = [ False ] * self._count
        stack = []
        for i in range(self._count):
            if not visited[i]:
                topo_sort_helper(self._graph, i, visited, stack)
        return stack

class IDMapper(object):
    def __init__(self):
        self._ids = {}
        self._items = {}

    def fetch_id(self, item):
        id = self._ids.get(item)
        if id is None:
            id = len(self._ids)
            self._ids[item] = id
            self._items[id] = item
        return id

    def fetch_item(self, id):
        return self._items[id]

class MappedGraph(object):
    def __init__(self):
        self._map = IDMapper()
        self._graph = Graph()

    def add_edge(self, u, v):
        u_id = self._map.fetch_id(u)
        v_id = self._map.fetch_id(v)
        self._graph.add_edge(u_id, v_id)

    def topo_sort(self):
        return [ self._map.fetch_item(id) for id in self._graph.topo_sort() ]
