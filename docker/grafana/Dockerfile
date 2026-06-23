FROM grafana/grafana-enterprise:latest

# Install the SQLite datasource plugin during image build
RUN grafana-cli plugins install frser-sqlite-datasource

# Ensure proper permissions
RUN chown -R grafana:grafana /var/lib/grafana
