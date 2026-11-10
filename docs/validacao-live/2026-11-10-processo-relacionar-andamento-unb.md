---
type: SEIObservation
title: "UnB de treinamento: processo relacionar e andamento, PASS com saída 0"
description: "Ensaio de 10/11/2026 da #258 e da #259: relacionar, desfazer e registrar andamento com saída 0 e releitura independente, depois de uma correção na leitura do script de remoção"
---

# Processo relacionar e andamento — UnB de treinamento, 10/11/2026

**Issues:** [#258](https://github.com/franklinbaldo/ies/issues/258) e
[#259](https://github.com/franklinbaldo/ies/issues/259).
**Origem:** somente `https://sei.treinamento.unb.br/sei` (UnB SEI 4.0.9 de
**treinamento**), conta pública `aluno1`, unidade `Unidade1`; sessão em
pasta `0700` com arquivos `0600`, fora do repositório.
**Processos:** `23106.001906/2026-14` (origem) e `23106.001860/2026-25`
(destino do relacionamento), ambos Gerados da Unidade1.
**Execução:** binário release de `main@a164f3f` mais a correção desta PR;
rede direta do sandbox. **Autorização:** pedido do proprietário nesta sessão.

## Orçamento

Declarado **60**; usado cerca de **51** requisições contadas pela CLI.
Cada `--confirmar` foi executado uma única vez, com trava `O_EXCL`; as duas
recusas do desfazer saíram com 1 **antes** do POST.

## Relacionar

| # | Comando | Resultado | Req. |
| --- | --- | --- | --- |
| 1 | `relacionar NUP --com DESTINO` (prévia) | PASS, destino `51674` resolvido | 4 |
| 2 | `… --confirmar` | **PASS saída 0**, `confirmado`, `alterado` | 6 |
| 3 | prévia de novo | `sem_efeito: true` (relação existe) | 3 |
| 4 | `… --remover` (prévia) | PASS | 3 |
| 5 | `… --remover --confirmar` | saída 1 antes do POST: `acao_excluir_relacionamento_nao_declarada` | — |
| 6 | idem, build local de diagnóstico | mesma recusa; HTML da lista gravado em arquivo `0600` fora do repositório | — |
| 7 | `… --remover --confirmar` (corrigido) | **PASS saída 0**, `confirmado`, `alterado` | 5 |
| 8 | prévia de relacionar | `sem_efeito: false` (relação desfeita) | 4 |

**Defeito corrigido.** A UnB envolve o script da lista em
`<!--//--><![CDATA[//><!--`. O tokenizador lê isso como comentário e
`inner_text()` omitia a função `acaoRemover`. `url_exclusao` passa a ler a
fonte bruta da tag (`raw()`), com teste que falha sem a correção e passa com
ela. O ensaio do passo 7 usou uma versão com `inner_html()`, que a guarda
`tests/atributos_html_pelo_tokenizador.rs` proíbe; a versão final, com
`raw()`, foi conferida offline contra o HTML real gravado no passo 6 (mesma
URL de exclusão), e o arquivo foi apagado em seguida por conter `infra_hash`. A forma do handler (`function acaoRemover(id){`, `confirm`, `action`,
`submit`) é a mesma já observada no PEN1. O build de diagnóstico não foi
commitado.

## Andamento

| # | Comando | Resultado | Req. |
| --- | --- | --- | --- |
| 1 | `andamento NUP --texto "Ensaio IES #259: …"` (prévia) | PASS, histórico com 2 itens | 4 |
| 2 | `… --confirmar` | **PASS saída 0**, `total_antes: 2`, `total_depois: 3` | 8 |
| 3 | prévia de novo | `total_antes: 3` | 4 |

## Resultado e resíduo

**PASS** para relacionar, desfazer relacionamento e registrar andamento.
Pela decisão do proprietário de 10/11/2026, as duas escritas ficam
aprovadas para produção (`src/origem_de_treinamento.rs`).
Resíduo: o andamento "Ensaio IES #259: andamento de teste em treinamento"
no histórico do processo `23106.001906/2026-14`, irreversível por natureza.
O relacionamento foi desfeito.
