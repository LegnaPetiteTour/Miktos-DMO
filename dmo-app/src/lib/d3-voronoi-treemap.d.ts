declare module "d3-voronoi-treemap" {
  export function voronoiTreemap(): VoronoiTreemapLayout;

  interface VoronoiTreemapLayout {
    (root: any): void;
    clip(polygon: [number, number][]): VoronoiTreemapLayout;
    minWeightRatio(ratio: number): VoronoiTreemapLayout;
    maxIterationCount(count: number): VoronoiTreemapLayout;
    convergenceRatio(ratio: number): VoronoiTreemapLayout;
  }
}
