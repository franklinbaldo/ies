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

A operação é escolhida pelo campo `request.operation` dos passos
verificados, nunca pelo nome da cassette. Sem `--passo`, o replay
seleciona a resposta GET `procedimento_consultar_historico` quando
observada; caso contrário, seleciona `procedimento_controlar`.
Histórico selecionado explicitamente com `--passo N` deve funcionar
mesmo em cassettes que também contêm outras operações, preservando
o NUP indicado no título e a projeção parcial de `Andamento`.

`--passo N` é um índice **zero-based** de interação da cassette.
A opção técnica `--todas-paginas` reúne automaticamente as interações
contíguas de `procedimento_controlar` da **mesma cassette verificada**:
primeira página GET e páginas subsequentes GET/POST capturadas. Não
executa POST nem consulta o SEI. A união passa pelas invariantes existentes
de unidade, filtros, faixas e totais. Operações intercaladas, ausência
de página inicial ou cobertura insuficiente devem falhar. É incompatível
com `--passo` e `--documento`. A opção só abrange o que foi gravado;
não descobre páginas ou garante snapshot transacional.

Vários passos são aceitos para consolidar páginas do Controle de Processos,
mas não se podem misturar operações nem compor um histórico artificial
a partir de respostas distintas. A seleção de passo ausente, operação
não reconhecida ou operações diferentes deve falhar com subcódigo
específico sem acessar a rede. O replay não é consulta live nem prova
de leitura integral do processo.

No histórico verificado, `--numero-processo NUP` exige correspondência
exata entre o número solicitado e o título do HTML, preservando zeros,
pontuação e demais caracteres opacos aceitos no contrato. Não deduz
identidade pelo nome da cassette. Entrada inválida resulta em saída 2
com erro tipado; número diferente resulta em saída 1 com subcódigo
`numero_do_historico_diferente_do_solicitado`. Sem a opção, preserva a
leitura do NUP observado no título. Não há consulta live ao SEI.

## Documento HTML capturado

`ies replay CENARIO --documento --json` seleciona somente a resposta
`documento_imprimir_web` da cassette verificada e extrai do título o
número SEI e o tipo, preservando zeros. Se houver mais de uma resposta,
use `--passo N` para selecionar um único HTML registrado. O recorte `--documento --numero-documento DOC` exige número SEI decimal
com zeros preservados e escolhe **somente** o título exato desse documento
entre todas as respostas `documento_imprimir_web` GET verificadas da
cassette; falha ao faltar a identidade, existir mais de uma resposta com
o mesmo número, ou aparecer título sem identificação SEI. Não escolhe
um documento por posição arbitrária nem por substrings. `--passo N`
continua disponível para limitar manualmente o escopo da consulta.
Não envia
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
