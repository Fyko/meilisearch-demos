import { instantMeiliSearch } from '@meilisearch/instant-meilisearch'
import injectScript from 'scriptjs'

const GOOGLE_API = process.env.GOOGLE_API

injectScript(
  `https://maps.googleapis.com/maps/api/js?v=quarterly&key=${GOOGLE_API}`,
  () => {
    const search = instantsearch({
      indexName: 'world_cities',
      searchClient: instantMeiliSearch(
        'https://demos.meilisearch.com',
        'L4h8HWpdac31f126b8eebbb23e2d10752e86d57d8a388e6910c2fedfb53542547dd4775c',
        {
          limitPerRequest: 200,
        }
      ),
    })

    search.addWidgets([
      instantsearch.widgets.sortBy({
        container: '#sort-by',
        items: [
          { value: 'world_cities', label: 'Relevant' },
          {
            value: 'world_cities:population:desc',
            label: 'Most Populated',
          },
          {
            value: 'world_cities:population:asc',
            label: 'Least Populated',
          },
        ],
      }),
      instantsearch.widgets.searchBox({
        container: '#searchbox',
      }),
      instantsearch.widgets.configure({
        hitsPerPage: 20,
      }),
      instantsearch.widgets.geoSearch({
        container: '#maps',
        googleReference: window.google,
        initialZoom: 7,
        initialPosition: {
          lat: 50.655250871381355,
          lng: 4.843585698860502,
        },
      }),
      instantsearch.widgets.infiniteHits({
        container: '#hits',
        templates: {
          item: `
            <div>
              <div class="hit-name">
                City: {{#helpers.highlight}}{ "attribute": "name" }{{/helpers.highlight}}
              </div>
              <div class="hit-name">
                Country: {{#helpers.highlight}}{ "attribute": "country" }{{/helpers.highlight}}
              </div>
              <div class="hit-name">
                Population: {{#helpers.highlight}}{ "attribute": "population" }{{/helpers.highlight}}
              </div>
            </div>
          `,
        },
      }),
    ])

    search.start()
  }
)
