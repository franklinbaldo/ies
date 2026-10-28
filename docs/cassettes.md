---
type: Guide
title: Catálogo das cassettes públicas
description: Localizar, conferir e reproduzir gravações de treinamento do SEI
---

# Cassettes do SEI

O catálogo `cassettes/` é **um único bundle OKF**, com um arquivo
Markdown para cada revisão distinta de cassette. O Git guarda apenas
os metadados verificáveis: todo ZIP e toda cassette JSON original
continuam no [Internet Archive](https://archive.org/details/todos-sei-cassettes-2026).

O inventário atual contém **25 ZIPs (7 históricos e 18 lotes)**, com
485 capturas e **478 revisões distintas**; sete cópias idênticas
foram consolidadas. Instalações: UnB, UFRJ, Goiás e PEN1–PEN4.

## Procedimento para agentes

1. Localize o cenário pelo nome, instalação ou operação em `cassettes/`.
   Por exemplo, `rg -l 'documento_gerar' cassettes/`, ou
   `uvx --from okf-parser==0.48.2 okf-parser search cassettes 'documento'`.
2. Leia o card. Os campos `source_zip` e `archive_member`
   indicam o arquivo exato. Não monte URLs a partir do nome do cenário.
3. Baixe o ZIP do Internet Archive. Confirme seu `archive_sha1`.
4. Abra **somente** `archive_member` no ZIP (sem extrair caminhos
   desconhecidos). Compare seu SHA-256 após CRLF→LF com
   `cassette_sha256_lf`.
5. Exija `schema_version: 1`, `complete: true` e origem numa das
   sete instalações permitidas. Faça replay **offline**.
6. Distinga o que a resposta mostrou daquilo que seria necessário
   provar sobre uma operação completa.

## Exemplo verificado: bloco no PEN2

```sh
curl -fL -o /tmp/pen2.zip \
  https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-pen2.zip
echo '32cd3f7feb5a4a80b3b363e9750b9ce0b593d2ff  /tmp/pen2.zip' | sha1sum -c -
python3 - <<'PY'
import hashlib, json, zipfile
with zipfile.ZipFile('/tmp/pen2.zip') as z:
    raw = z.read('bloco-ciclo-pen2.json')
assert hashlib.sha256(raw.replace(b'\r\n', b'\n')).hexdigest() == (
    '1b3b82ded33b541ecc9396edfad6ae5a6137073d0a02168522e330d2fd0ef677'
)
cassette = json.loads(raw)
assert cassette['schema_version'] == 1 and cassette['complete'] is True
print(len(cassette['interactions']), 'interações registradas')
PY
```

## Limites da evidência

O corpus histórico preserva metadados da requisição e respostas
observadas, **mas não todos os corpos das requisições POST**.
O replay pode demonstrar layouts, paginação, HTML, mensagens e
respostas; ele **não prova sozinho paridade de escrita**.
HTTP 200 não significa efeito realizado. Para escritas, exija
conferência posterior do estado, em cenário testável.

Jamais grave ou publique capturas de produção, credenciais, cookies,
URLs assinadas, dados pessoais ou HTML autenticado no Git ou em logs.
Um cenário novo gera novo card e, se necessário, novo ZIP imutável
no Archive. Não substitua a captura histórica.

Leia também [como o SEI funciona](como-o-sei-funciona/README.md)
e o [contrato da CLI](../contracts/cli.md).
