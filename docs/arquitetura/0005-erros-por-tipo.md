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

## Consequências

- Erro novo: `#[derive(Causa)]`; nunca `fn subcodigo()` manual.
- Tabelas `subcodigos-*.md` deixam de ser a fonte; caem quando os
  contratos migrarem.
- `?` com `From` gerado para variantes `interna` de tipo único é o
  próximo passo do derive.
