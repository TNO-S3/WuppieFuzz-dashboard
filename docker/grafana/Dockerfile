FROM grafana/grafana-enterprise:latest

USER root

# Install the SQLite datasource plugin from the Grafana API during image build.
# See: https://grafana.com/docs/grafana/latest/administration/plugin-management/plugin-install/#install-a-plugin-in-air-gapped-environment
RUN apk add --no-cache unzip \
    && mkdir -p /var/lib/grafana/plugins \
    && wget -q -O /tmp/frser-sqlite-datasource.zip \
        "https://grafana.com/api/plugins/frser-sqlite-datasource/versions/4.0.6/download" \
    && unzip /tmp/frser-sqlite-datasource.zip -d /var/lib/grafana/plugins/ \
    && rm /tmp/frser-sqlite-datasource.zip

# Ensure proper permissions
RUN chown -R 472:472 /var/lib/grafana
