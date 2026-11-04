---
type: CommandContract
title: Buscar processo pela pesquisa Web do SEI
operation: processo.buscar
command: ies processo buscar TERMO [--pagina N] [--json]
effect: read
implementation: partial
source_commit: fffbc5abd07d44ccef79b3c0e6872e0bdc39c18b
---

# Busca de processo pela pesquisa Web (Refs #58, #56)

Sem relogin automático e sem REST/SOAP, `ies processo buscar TERMO
[--pagina N] [--json]` executa a pesquisa Web por protocolo com a sessão
dos arquivos privados gravados por [`ies login`](sessao-login.md)
(`IES_ORIGEM_FILE`, `IES_COOKIE_FILE`, `IES_URL_CONTROLE_FILE` ou
`IES_CONFIG_FILE`). O recorte offline `ies processo NUP
--resultado-pesquisa-file` foi aposentado em 04/11/2026 (decisão do
proprietário, [processo.obter](processo-obter.md)); respostas capturadas
seguem lidas por `ies replay --resultado-pesquisa`.

O stub anterior `ies processo buscar [--assunto]` foi substituído: o
verbo continua `buscar`, agora com `TERMO` posicional obrigatório e
`--pagina N`. `--assunto` deixou de existir (nunca foi implementado).

## Entrada

`TERMO` é texto opaco (NUP inteiro ou parte do número), com as regras de
`NumeroProcesso`: controles recusados antes de aparar espaços, vazio e
`sei://` recusados, saída 2 (`entrada_invalida`) **antes** de ler a
configuração. `--pagina N` é inteiro ≥ 1. Opções de leitura de
`ies processo NUP` (`--sem-documentos`, `--andamentos-completos`…) junto com
`buscar` são `opcoes_incompativeis_com_busca` (saída 2). `--unidade` é
pré-condição conferida na tela do Controle, não rótulo ecoado.

## Fluxo e orçamento

1. **GET** do Controle (URL assinada do arquivo, origem conferida antes do
   cookie); o Controle precisa ser reconhecido em qualquer visão (R ou D).
2. Seguir **somente** o link do menu que o Controle publicou com
   `acao=protocolo_pesquisar` (relativo `controlador.php?`, exatamente um
   `infra_hash` de 64 hex, mesma instalação, mesma resposta e geração).
   Ausente, repetido, absoluto ou fora da forma é recusa tipada sem rede.
3. **GET** da tela de pesquisa (`frmPesquisaProtocolo`, unidade igual à do
   Controle).
4. **POST** derivado como na [decisão](../../docs/decisoes/processo-buscar.md):
   ocultos como servidos, `rdoPesquisarEm` = valor de `#optProcessos`,
   `selOrgaoPesquisa[]` = opções servidas selecionadas, `sbmPesquisar` com
   o valor servido e `txtProtocoloPesquisa=TERMO`.
5. Com `--pagina N` (N > 1) numa lista: **um POST** de `navegar(inicio)`
   derivado da tela de resultado (ver Paginação).

Orçamento: página 1 = **GET + GET + POST** (3); página N = **GET + GET +
POST + POST** (4). Sem retry, sem redirect seguido, sem restaurar página ou
filtro (AGENTS.md, "Não restaurar a tela"). Efeito:
`AlteraEstadoDaSessao::PaginaOuFiltroLembrado`.

## Saídas (JSON, saída 0)

Todas têm `resultado`, `termo` (como digitado, aparado), `unidade` (do
Controle), `origem: "web"` e `requisicoes`.

- **302** com `Location` para `procedimento_trabalhar` (interpretado,
  nunca seguido): `{"resultado":"processo_unico","processo":{"id":"…"}}`.
  O 302 só traz o `id_procedimento` interno; o número formatado **não** é
  inventado a partir do termo.
- **200 com `table.pesquisaResultado`**: `{"resultado":"lista",
  "total_declarado":N,"faixa_observada":{"inicio":I,"fim":F},
  "processos":[{"numero":"…"}]}`. Total e faixa vêm da barra
  `div.pesquisaBarraD` "Exibindo I - F de N" ou, quando a lista cabe numa
  página, "N resultados" (faixa `1 … N`, sem página 2); o singular não foi
  observado e falha fechado; o separador de milhar `.`
  só é aceito entre grupos de 3 dígitos (`1.234`), qualquer outra forma
  falha fechada. As linhas lidas devem ser `F - I + 1`.
- **200 com `div.pesquisaSemResultado`** (sem tabela):
  `{"resultado":"sem_resultado","total_declarado":0,"faixa_observada":null,
  "processos":[]}`. É uma **resposta observada**, não erro e **não**
  `processo_inexistente`: significa "a pesquisa não devolveu nada para esta
  conta/unidade", sujeito a permissões e restrições de acesso.

## Erros tipados (saída 1, `fonte_invalida` com `subcodigo`)

- `validacao_recusada_pelo_sei`: 200 com `txaInfraValidacao`; publica
  `mensagem_do_servidor` saneada.
- `resposta_da_pesquisa_nao_reconhecida`: 200 sem tabela, sem aviso de
  "sem resultado" e sem validação; também tabela e aviso juntos.
- `aviso_de_sem_resultado_e_barra_de_contagem_na_mesma_resposta`: aviso de
  "sem resultado" com barra "Exibindo I - F de N" é ambíguo e não vira
  `sem_resultado`.
- Termo (antes de qualquer POST): `termo_acima_do_maxlength_da_tela`
  (mais unidades UTF-16 que o `maxlength` servido em
  `txtProtocoloPesquisa`), `maxlength_do_termo_ilegivel_na_tela`,
  `campos_do_termo_repetidos_na_tela`. Sem `maxlength` publicado, a tela
  não limita o termo.
- Link: `link_da_pesquisa_ausente_no_controle`,
  `links_da_pesquisa_repetidos_no_controle`,
  `link_da_pesquisa_com_endereco_absoluto`,
  `link_da_pesquisa_nao_reconhecido`.
- Barra: `barra_de_contagem_da_pesquisa_ausente`,
  `barras_de_contagem_da_pesquisa_repetidas`,
  `texto_da_barra_de_contagem_fora_do_formato`,
  `numero_da_barra_de_contagem_fora_do_formato`,
  `faixa_da_barra_de_contagem_impossivel`,
  `linhas_da_tabela_diferentes_da_faixa_da_barra`.
- Paginação: `pagina_fora_do_intervalo` (antes do POST de página),
  `regra_de_navegacao_da_pesquisa_nao_observada`,
  `paginador_da_pesquisa_nao_publicado`,
  `link_de_paginacao_da_pesquisa_fora_da_regra`,
  `protocolo_da_tela_de_resultado_diferente_do_pesquisado`,
  `redirecionamento_ao_paginar_a_pesquisa`,
  `pagina_da_pesquisa_sem_resultado_apos_paginar`,
  `total_da_pesquisa_alterado_apos_paginar`,
  `faixa_da_pesquisa_diferente_da_pagina_solicitada`.
- Além das recusas existentes do formulário (`regras_formulario`,
  `ErroSubmissaoFormulario`), do 302 (`ErroRedirecionamentoDaPesquisa`),
  das linhas (`ErroResultadoPesquisa`) e do transporte (`status_http`).

## Paginação (regra observada)

O resultado publica `function navegar(inicio)`, que faz
`document.getElementById('hdnInicio').value = inicio;`, chama
`onSubmitForm()` e `document.getElementById('frmPesquisaProtocolo').submit();`
(sem botão no corpo), e links `javascript:navegar('K')` com K múltiplo do
tamanho da página. O IES exige essas duas linhas literais e todo link
`navegar('K')` com K decimal, múltiplo do tamanho e menor que o total.

- Tamanho da página = `F - I + 1` da barra da página 1 quando `F < N`
  (observado: 10). Páginas = ⌈N / tamanho⌉; com `F = N`, uma página.
- `--pagina 1` não envia POST extra. N > páginas (inclusive em
  `sem_resultado`, 0 páginas, e `processo_unico`, 1 página) é
  `pagina_fora_do_intervalo` **antes** do POST de página.
- O POST de página deriva da tela de resultado (mesmo formulário, termo
  servido igual ao pesquisado), sem `sbmPesquisar`, com
  `hdnInicio = (N - 1) × tamanho`. A resposta precisa ser lista com o
  mesmo total e faixa `inicio = hdnInicio + 1`,
  `fim = min(hdnInicio + tamanho, N)`.

## Links das linhas do resultado

Em cada página, cada linha da tabela tem um `a.protocoloNormal` com
`procedimento_trabalhar`; a UnB 4.0.9 publica também um `a.arvore` com o
**mesmo** `href` e repete o atributo `class` idêntico no
`a.protocoloNormal`. Ambos são aceitos só nessa forma; `href` diferente,
mais de um `a.arvore` por linha ou `class` repetido com valores diferentes
falham fechado.

## Evidência e validação

- [Matriz Todos](../evidencias/processo-buscar.md) e ensaio estrutural da
  UnB 4.0.9 de 03/11/2026 (capturas privadas, só estrutura).
- [Parser e fluxo](../../src/pesquisa_web.rs), com testes de sequência de
  requisições por transporte injetável e
  [superfície da CLI](../../tests/processo_buscar_web.rs).
- UnB live do comando IES: **NOT_RUN** até o ensaio do operador;
  UFRJ/GO/PEN: **NOT_RUN**.
