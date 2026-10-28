---
type: Guide
title: Primeira leitura da caixa do SEI
description: Replay com cassettes verificadas e GET seguro de uma caixa já autenticada
---

# Primeira leitura de processos da unidade

O IES Rust interpreta duas tabelas independentes da caixa SEI:
`tblProcessosRecebidos` e `tblProcessosGerados`. O resultado JSON
contém número, identificador interno, visualização, total declarado,
linhas lidas e `completude` de cada categoria.

**Esta etapa implementa a primeira página observada.** Não tenta
paginar nem inventa que uma lista está completa quando o total é maior
que as linhas lidas. Não inclui login, troca de unidade ou escrita.

## Replay offline de cassette do Internet Archive

Use o [card OKF de Goiás](../cassettes/go/unidade-processos-paginado-go--ca25324d2aac.md)
e baixe o ZIP que ele referencia. Confira o SHA-1 antes de executar:

```sh
curl -fL -o /tmp/ies-go.zip \
  https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
echo '42a08cd3714bb863d136ed98e517221510a31982  /tmp/ies-go.zip' | sha1sum -c -
cargo run --locked -- unidade listar-processos \
  --cassette-card cassettes/go/unidade-processos-paginado-go--ca25324d2aac.md \
  --zip-file /tmp/ies-go.zip --json
```

O IES também confere internamente SHA-1 do ZIP, SHA-256
normalizado da cassette, host público permitido, schema e status 200.
Não publica cookies, HTML bruto nem URLs `infra_hash`.

## Leitura HTTPS de caixa assinada

Para uma sessão **já autenticada**, o IES aceita exclusivamente
GET de URL HTTPS publicada pela própria caixa do SEI, assinada na
mesma sessão. Forneça um arquivo privado `0600` com os pares
da linha `Cookie` (por exemplo, `PHPSESSID=...; OUTRO=...`),
sem prefixo `Cookie:`.

```sh
chmod 600 /caminho/privado/cookies /caminho/privado/url-caixa
ies unidade listar-processos \
  --url-caixa-file /caminho/privado/url-caixa \
  --cookie-file /caminho/privado/cookies --json
```

O exemplo de URL é ilustrativo: **não construa nem adivinhe o hash**.
Obtenha-o intacto no navegador ou cliente autorizado. O programa
não segue redirects, não faz POST, não cria login e não repete
pedidos automaticamente. Falhas não imprimem cookies ou URL assinada.

Em arquivos `--saida`, a escrita usa modo `0600` em Unix,
recusa sobrescrever outro arquivo e produz apenas o resultado
estruturado. Os dados da caixa podem ser sensíveis: não envie
essas saídas ao Git ou ao Internet Archive.

## Testes e limites

- UnB: 25 recebidos e 18 gerados; ambas as tabelas completas.
- PEN3: 100 recebidos de 168 e 91 gerados de 91; parcial.
- Goiás: 10 recebidos de 131 e 10 gerados de 7939; parcial.

Os três resultados foram reproduzidos offline a partir dos ZIPs
públicos, sem gravações no repositório. O `cargo test` utiliza
HTML sintético para proteger parser e transportes contra regressões.

Próximos passos: tratamento de paginação conforme o formulário da
instalação, fluxo autenticado sem tokens em comandos e validação das
listagens por cursores e critérios.
