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

## Documento HTML capturado

`ies replay CENARIO --documento --json` seleciona somente a resposta
`documento_imprimir_web` da cassette verificada e extrai do título o
número SEI e o tipo, preservando zeros. Se houver mais de uma resposta,
use `--passo N` para selecionar um único HTML registrado. Não envia
requisições, não expõe o HTML integral e **não equivale** a
`ies documento NUMERO`: conteúdo e propriedades ainda não foram lidos.

Exemplo com a cassette pública `documento-numero-e-busca-unb`,
que contém a resposta GET da operação. O ZIP indicado no card OKF
precisa estar no cache local; sem ele o replay falha fechado.

## Consulta Web autorizada da unidade

O comando de domínio é `ies unidade listar-processos`. A sessão
existente é informada por caminhos de arquivos privados no ambiente:

```sh
IES_URL_CONTROLE_FILE=/privado/url-caixa \\
IES_COOKIE_FILE=/privado/cookies \\
IES_ORIGEM_FILE=/privado/origem \\
ies unidade listar-processos --json
```

A consulta exige HTTPS e valida host/caminho-base antes de enviar
cookies. Não segue redirects nem executa login, POST ou escrita.
Se o Controle de Processos exige outras páginas, o comando falha
em vez de apresentar uma caixa parcial como resultado integral.

Veja [cassettes](cassettes.md),
[contrato de processo](../contracts/cli/processo-obter.md) e
[primeira leitura da caixa](unidade-primeira-leitura.md).
