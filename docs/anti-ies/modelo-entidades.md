---
type: Guide
title: Modelo incremental de entidades e eficiência IES ↔ anti-IES
description: Identidades, observações, projeções SQLite e métricas do primeiro circuito
---

# Entidades, observações e eficiência

O IES **lê o SEI**, interpreta a resposta e alimenta o SQLite. O
anti-IES **lê o SQLite** e renderiza o comportamento observável
daquele SEI para testes. Não transformar o banco numa coleção de
HTMLs/JSONs, nem tentar modelar todas as operações antes de conhecer
seus fatos e relações. Ver [SQLite companion](sqlite-companion.md),
[roadmap](roadmap.md) e [#360](https://github.com/franklinbaldo/ies/issues/360).

## Biblioteca comum: `iesei`

A crate `crates/iesei` será importada **diretamente por `ies` e
`anti-ies`**. Nela moram identidades e entidades de domínio,
observações tipadas, proveniência, cobertura de listas, invariantes
e o contrato versionado de persistência/consulta SQLite. O IES
aciona a gravação de fatos obtidos do SEI; o anti-IES usa consultas
tipadas read-only. A biblioteca não precisa de um processo servidor.

Já existe `crates/ies-core`, atualmente com `Instalacao`.
**Migrar esse conteúdo para `iesei` e aposentar `ies-core`**
na [#360](https://github.com/franklinbaldo/ies/issues/360),
sem manter dois domínios paralelos. `ies-derive` é uma macro
procedural e pode continuar separada.

**Não compartilhar implementações de fronteira:** o parser HTML,
HTTPS, sessão e CLI são do IES; o roteador HTTP, HTML gerado e
estado ficcional são do anti-IES. O crate comum não pode depender
dos binários. Medir o custo de rebuild incremental e documentar
a migração, conforme [#214](https://github.com/franklinbaldo/ies/issues/214).

## Três camadas diferentes

1. **Entidade de domínio:** identidade durável de instalação, unidade,
   processo e documento, depois andamentos, usuários, assinaturas e
   outros fatos. Identificador opaco e nome visível não são iguais;
   preservar zeros à esquerda e escopos por instalação.
2. **Observação:** o que a tela de uma operação efetivamente afirmou,
   em qual instalação, identidade/contexto, instante e versão,
   com proveniência para cassette/ensaio. Uma lista parcial **não**
   prova nem completude nem ausência das demais linhas.
3. **Estado ficcional da bancada:** efeitos de operações simuladas
   posteriormente pelo anti-IES. Nunca alterar fatos históricos
   coletados pelo IES para fingir uma operação real.

Os tipos Rust devem expressar distinções relevantes, como
`Desconhecido`, `PaginaObservada` e `ListaCompleta`.
O `Option` representa ausência de dado quando só isso é conhecido,
não uma negativa observada pelo SEI. Não usar strings com palavras
especiais para classificar erro, estado, categoria ou completude.
Não criar enum ou validador onde não há invariante demonstrada.

## Primeiro recorte: unidade.atual

Começar com `Instalacao`, `ContextoDeObservacao`,
`UnidadeExibida`, `VisaoDoControle` (R/D quando observada),
`ObservacaoDoControle`, `Proveniencia` e
`CoberturaDaLista`. Os processos que o Controle realmente
publicar podem usar os tipos já existentes de
[`src/dominio.rs`](../../src/dominio.rs), sem duplicá-los
num "domínio anti-IES". Unidade exibida **não** dá automaticamente
o ID administrativo da unidade nem a lista de permissões.

Exemplo **ilustrativo**, não esquema SQL já instalado:

```text
ObservacaoDoControle {
    origem: instalacao_sintetica,
    contexto: identidade_sintetica,
    unidade_exibida: "PPI",
    visao: R,
    recebidos: PaginaObservada(total=12, faixa=1..10),
    gerados: Desconhecido,
    fonte: cassette_validada,
    observada_em: instante_observado
}
```

Nesse exemplo o anti-IES **não pode servir** um Controle R
válido com a tabela Gerados marcada como zero. Deve recusar a
resposta completa até haver observação suficiente; não inventar
`lnkInfraUnidade` isolado para que o parser aceite.

## Persistência e consultas

Evoluir o `ies.sqlite3` já existente com migração versionada,
sem copiar o modelo inteiro do Todos. Separar a projeção de domínio
das tabelas de sessão (particularmente `link_assinado`).
Para a primeira operação, identificar as consultas que o servidor
precisa fazer **antes** de escolher as tabelas e índices:
selecionar observação coerente por instalação/contexto, recuperar
a unidade e as tabelas/linhas de uma visão R ou D.

Preferir tabelas relacionais específicas e índices pelas chaves
efetivamente consultadas; usar transação curta por observação/lote,
`INSERT ... ON CONFLICT` quando a semântica de substituição for
demonstrada, e operações idempotentes. `SELECT` preparado e leitura
em conjunto para evitar **N+1**; não abrir uma consulta por processo
da caixa. Evitar camada ORM genérica, EAV universal, duplicação de
serializações e persistência obrigatória de HTML completo.
As cassettes continuam sendo evidência, não armazenamento de domínio.

Não requerer daemon anti-IES para o IES persistir. O cliente segue
funcionando se o SQLite for desativado/corrompido, sem esconder erro
de consistência da projeção usada pelo servidor. O anti-IES de teste
abre somente a projeção autorizada (ou banco temporário sanitizado).
Nunca publicar cookies, `infra_hash` ou URLs assinadas do índice.

## Benchmark desde o início

Comparar **mesmo ambiente e mesmos dados** antes/depois de cada fatia,
com casos sintéticos representando página pequena, intermediária e
grande (por exemplo, 1, 1.000 e 10.000 registros quando pertinentes).
Contar e reportar, **conforme a camada estiver implementada**:

| Medida | O que demonstrar |
| --- | --- |
| Ingestão | tempo de parse e de upsert em lote, primeira e segunda gravação |
| Leitura SQLite | número de consultas por GET, latência p50/p95, plano com `EXPLAIN QUERY PLAN` |
| Servir HTTP (P2/P3) | latência p50/p95 do GET em loopback, tamanho do HTML, throughput quando fizer sentido |
| Recursos | pico de memória/RSS, tamanho do arquivo SQLite, crescimento por lote |
| Corretude/custo remoto | resultado e recusas iguais ao oráculo, **zero requests adicionais ao SEI** |

Não impor alvo arbitrário em milissegundos sem baseline nem medir
tempo de rede do SEI como se fosse latência do anti-IES. Usar
benchmark isolado de parse, persistência, consulta e HTTP, sem
sacrificar tipos ou semântica para atingir números irrelevantes.
Na P1.2, a baseline cobre parse/projeção e consultas SQL de bancada;
a latência HTTP da capacidade só é medida nas P2/P3, quando
houver renderizador funcional. O endpoint de saúde não é substituto.
Gates determinísticos: orçamento de requisições, não ter N+1
observável, idempotência e integração E2E. Métricas de latência
podem começar como relatório, não como teste sensível à máquina do CI.

## Sequência e extensão

- [#353](https://github.com/franklinbaldo/ies/issues/353):
  oráculo/cassettes da primeira leitura.
- [#360](https://github.com/franklinbaldo/ies/issues/360):
  modelo mínimo tipado, esquema/projeção proposta e benchmark base.
- [#358](https://github.com/franklinbaldo/ies/issues/358):
  IES registra a observação no SQLite.
- [#354](https://github.com/franklinbaldo/ies/issues/354):
  anti-IES consulta e renderiza o Controle.
- [#355](https://github.com/franklinbaldo/ies/issues/355):
  integração real IES → SQLite → anti-IES → IES.

Depois de `unidade.atual`, ampliar para
`unidade.listar_processos`, `processo.obter` e
`documento.obter`. Só introduzir tabelas/tipos quando uma
capacidade e suas cassettes exigirem esses fatos; conservar
migrações e compatibilidade de leitura. Relacionar as medições
com o [benchmark IES × Todos #95](https://github.com/franklinbaldo/ies/issues/95)
sem confundir comparações de parser com cobertura semântica.
