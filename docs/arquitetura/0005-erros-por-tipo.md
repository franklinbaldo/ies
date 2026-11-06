---
type: ArchitectureDecision
title: Erros por tipo composto, com causa derivada e texto só na fronteira
description: derive Causa local declara a identidade pública ao lado de onde o erro é levantado e substitui as fn subcodigo manuais
status: accepted
decision: derive-causa compose-by-type text-at-boundary no-uncertain-effect
---

# 0005 — Erros por tipo composto

## Problema

Cada `enum` de erro mantinha uma `fn subcodigo()` manual: 966 braços de
`match` que copiavam a classificação tipada para texto. O compilador
garante que o `match` cobre as variantes, mas não que o texto corresponda
à variante. Por isso existiam tabelas de subcódigos longe do código e um
hook só para mantê-las em sincronia. Nenhum consumidor programático decide
pelo `subcodigo`: os únicos usos eram testes e o próprio verificador.

## Decisão

- **O tipo é a verdade.** O erro é declarado no módulo que o levanta e
  deriva `Causa` (`crates/ies-derive`, sem dependências externas). O nome
  público é o `snake_case` da variante; um nome histórico divergente fica
  explícito em `#[causa(nome = "...")]`.
- **Compor em vez de nomear o contexto.** `#[causa(interna)]` encadeia o
  erro interno: a causa vira um caminho (`["abertura", "transporte",
  "falha_de_rede"]`), e o contexto vem da estrutura, não do nome.
- **Uma etapa, um tipo.** Um `enum` não tem duas variantes com o mesmo
  tipo interno: isso é duas etapas achatadas num nível, e o nome da
  variante passa a carregar a etapa. Cada etapa com mais de uma fonte de
  falha ganha o próprio tipo, e a árvore de erros espelha a de chamadas.
  Assim o derive gera um `From` por variante `interna` e o `?` compõe sem
  `map_err`.
- **Decisão separada de diagnóstico.** `Acao` (corrigir argumentos,
  relogar, trocar de instalação, conferir a escrita, reportar) é o
  contrato de decisão; a causa é diagnóstico.
- **Sem "efeito incerto".** Uma escrita é descrita por fatos (`enviada`,
  `aplicada`) e pela causa exata; depois de uma falha, a operação lê de
  novo para reconciliar, sem repetir a escrita.
- **Texto só na fronteira.** `src/erro_publico.rs` serializa. Nenhum
  código compara o texto de um erro: classifica-se por `match`, e os
  testes casam o tipo.

## Migração

`scripts/migrar_causas.py` (tree-sitter, fora do gate) converteu 83 tipos
cujos braços eram literais ou delegações, sem mudar nenhum nome público.
A suíte de contrato é a prova. Os oito tipos restantes têm braços não
mecânicos e migram à mão, e o piloto de `acao`/`escrita` é `processo
concluir`. O campo `subcodigo` segue publicado (folha do caminho) até os
contratos passarem a `acao` e `causa`.

Segunda passada: 10 tipos mais (inclusive `ErroArvore`, cujo `match`
estava partido em duas funções, e a cascata `ErroProcessoObter` →
concluir, reabrir, enviar, assinar), e `scripts/migrar_interrogacao.py`
trocou 152 `.map_err(Enum::Caso)?` por `?`. Só troca quando a função
envolvente devolve o próprio `Enum` e o `?` não está numa closure.

Restam à mão:
- **Variantes com `&'static str`** (o leitor devolve o subcódigo como
  texto): `FalhaAnexar::Estrutura`, `FalhaRelacionar::Estrutura`,
  `ErroCriar::Transporte`, e o padrão aninhado de
  `processo_andamento_registrar::Erro`.
- **Etapas achatadas** (tipo interno repetido): `ErroLogin` e
  `ErroReabrir` (`FalhaTransporte`), `ErroProcessoObter`
  (`ErroSubmissaoFormulario`).

## Forma não observada

Um elemento ausente, repetido, com valor fora da forma ou um HTML
ilegível não sustenta decisão: só diz onde a leitura da tela falhou.
Por isso é um tipo só, `ErroHtml { onde, problema, mensagem }`
(`src/erro_html.rs`), e não uma variante por leitor.

- `onde` é o diagnóstico e a folha pública: o `subcodigo` legado não
  muda. `problema` (`ausente`, `repetido`, `valor`, `html`, ou
  `atributo` com o `ErroAtributoHtml` que subiu) vai no caminho, depois
  de `html`.
- A mensagem é a genérica do problema (`ErroHtml::ausente("onde")`);
  `.com_mensagem("...")` só onde o ponto tem texto próprio. A igualdade
  compara `onde` e `problema`, não a mensagem.
- Um `enum` com outras causas compõe com `#[causa(transparente)]
  Html(ErroHtml)`: o `From` derivado deixa `?` e `.into()` converterem, e
  o caminho não repete `html`. Um leitor que só falha por forma devolve
  `ErroHtml`.
- A variante de forma que algum código casa fora de teste (`match` em
  fluxo, `matches!`, `==`, conversão entre tipos) sustenta uma decisão e
  continua sendo variante.
- **Leitor novo não declara `enum` de forma: devolve `ErroHtml`.**

`scripts/migrar_erro_html.py` (tree-sitter, com o compilador como
oráculo) migrou 350 variantes de 68 tipos e eliminou 4 tipos inteiros. Os
77 `map_err(|_| ...)` cuja origem era `ErroAtributoHtml` passaram a
carregar a causa. As 21 variantes casadas fora de teste ficaram.

## Consequências

- Erro novo: `#[derive(Causa)]`; nunca `fn subcodigo()` manual.
- Tabelas `subcodigos-*.md` deixam de ser a fonte; caem quando os
  contratos migrarem.
- `?` em vez de `map_err` para compor causas; um `From` manual só onde
  a origem não é um erro do projeto (`io`, `rusqlite`).
