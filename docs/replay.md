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

## Inventário offline de documentos observados

`ies replay CENARIO --documento --listar-documentos --json` lista os
números SEI e tipos de todas as respostas `documento_imprimir_web`
verificadas da cassette, na ordem da gravação, sem mostrar conteúdos.
Não deduz documentos que não apareceram na evidência. Recusa documentos
sem título SEI identificável, números repetidos (observações não são
identidades únicas) e cassettes que não atendam à verificação de hashes.
A lista é uma observação offline, não uma busca live nem uma árvore completa
de documentos. A opção não pode ser combinada com `--passo` ou
`--numero-documento`.

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

## Texto de documento já verificado

`ies replay CENARIO --documento --numero-documento DOC --texto-documento --json`
expõe somente o texto visível do `body` do HTML selecionado e hash-verificado,
separado do título, sem executar rede ou scripts. Recusa corpo ausente, vazio,
script/estilo executável no corpo e texto acima do limite; URLs assinadas em
atributos HTML não são serializadas. O conteúdo pode conter dados pessoais:
tratar a saída como informação confidencial e não registrar em logs de CI.
Sem `--texto-documento`, a saída continua limitada à identidade/tipo, sem
expor corpo. Isto **não é** a leitura live `ies documento DOC`.

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


## Busca textual nos documentos já capturados

`ies replay CENARIO --documento --termo-documento TERMO --json` consulta
somente textos de documentos `documento_imprimir_web` cujos ZIPs e cards
foram verificados. A busca usa caracteres Unicode sem distinção de caixa,
retorna apenas número/tipo dos documentos observados e **não** publica o
corpo nem faz GET/POST no SEI. Termo vazio, caracteres de controle,
documento duplicado ou corpo impossível de interpretar são falhas
tipadas. Nenhum achado significa apenas ausência entre os documentos
**da cassette escolhida**, não inexistência no SEI. Incompatível com
`--passo`, `--numero-documento`, `--texto-documento` e
`--listar-documentos`. A busca live de documentos segue indisponível.

## Resultados de uma pesquisa de processos capturada

`ies replay CENARIO --resultado-pesquisa --passo INDICE --json` interpreta
**uma resposta POST 200** de `protocolo_pesquisar` já presente em uma
cassette pública com ZIP, SHA-1, SHA-256, instalação e MIME verificados.
O índice é zero-based da interação gravada, não a ordem dos resultados.
Exemplo observado: `processo-busca-unb--67c8f2ed35b6`, `--passo 2`, UnB.

A saída devolve somente os NUPs observados nos links de processo da tabela
`pesquisaResultado`, preservando zeros e pontuação, sem URLs assinadas,
IDs internos, trechos de documentos ou HTML. Um formulário GET não pode
ser confundido com resultado. O leitor não executa links nem novas
requisições ao SEI, não reproduz formulários e não oferece busca live.
Entrada de passo inválida, ausência ou multiplicidade de tabelas de
resultados, linha com múltiplos links, ação inesperada, link sem identificador
interno de processo, identificador interno que não seja decimal ASCII,
parâmetro de identificador repetido, ação com prefixo `amp;` no primeiro
parâmetro (não é uma ação `acao` real), chaves de query não canônicas
(como `%5F`, `+` ou ponto, que o servidor PHP pode normalizar para `_`),
dois NUPs distintos apontando ao mesmo
identificador interno (inclusive representações decimais equivalentes como `7` e
`007`) e NUP duplicado geram erros específicos. O identificador interno é conferido apenas na origem;
nunca é incluído no resultado JSON. Ausência de achados significa apenas
ausência na resposta escolhida, jamais inexistência no SEI. Ver a
[matriz da operação](../contracts/evidencias/processo-buscar.md);
a capacidade live segue pendente na issue #58.
