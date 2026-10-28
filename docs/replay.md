---
type: Guide
title: Replay de cenários SEI
description: Interface de reprodução de evidências separada dos comandos de domínio
---

# Replay de cenários

Os comandos `ies processo NUP`, `ies documento NUMERO` e
`ies unidade listar-processos` não aceitam cards, ZIPs, passos nem
URLs de sessão. O replay é uma ferramenta separada que utiliza os
mesmos parsers e tipos de domínio, **sem rede e sem efeito no SEI**.

## Uso

```sh
ies replay processo-andamento-go --json
ies replay unidade-processos-paginado-go --json
```

O nome do cenário é localizado automaticamente sob `cassettes/`,
comparando o nome completo do card ou seu prefixo anterior a `--hash`.
Se existirem várias revisões compatíveis, a execução recusa
ambiguidade; use o nome mais específico. Use `--acervo DIR` quando
estiver executando fora da raiz do repositório.

O ZIP referenciado no card deve estar no cache local
`.cache/ies/<nome-original-do-zip>`, preservando seu basename.
O replay **não realiza download automático**: a ausência do arquivo
produz erro claro e não inicia rede. O card define a origem exata;
não montar URLs a partir do nome do cenário. Exemplo de preparação:

```sh
mkdir -p .cache/ies
curl -fL -o .cache/ies/todos-sei-cassettes-go.zip \
  https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
ies replay processo-andamento-go --json
```

O leitor verifica SHA-1 do ZIP, SHA-256 da cassette normalizada,
`schema_version`, completude declarada, host permitido, operação,
status HTTP e codificação antes de analisar HTML. Nenhum corpo de
requisição POST é enviado ou repetido.

## Seleção de respostas

Para cenários de andamento, a ferramenta seleciona a única resposta
GET `procedimento_consultar_historico`, confere o número no título
e retorna uma projeção parcial de `Andamento`.
Para cenários de caixa, seleciona a resposta
`procedimento_controlar`; com múltiplas caixas, requer seleção
explícita. Somente o comando técnico `ies replay` aceita `--passo N`
repetido para consolidar páginas já registradas.

A inferência atual de categoria usa o nome do cenário contendo
`andamento`; outros cenários tentam caixa. Trata-se de um recorte
conservador: cenários de outras operações ainda não possuem replay
de domínio e falham em vez de inventar uma interpretação.

## Sessão Web experimental

O acesso Web com uma sessão existente foi separado da CLI de domínio:

```sh
ies sessao ler-caixa \
  --url-caixa-file /privado/url-caixa \
  --cookie-file /privado/cookies \
  --origem-file /privado/origem --json
```

Essa operação exige HTTPS, host e caminho-base conferidos antes do
envio de cookies e não segue redirects. Não executa login, POST,
paginação ou comandos de escrita. Abrir um processo pode registrar
recebimento no SEI, portanto não generalizar a segurança dessa
leitura da caixa para outras páginas.

Veja [cassettes](cassettes.md),
[contrato de processo](../contracts/cli/processo-obter.md) e
[primeira leitura da caixa](unidade-primeira-leitura.md).
