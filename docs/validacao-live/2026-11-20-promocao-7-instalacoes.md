---
type: SEIObservation
title: "Promoção de seis escritas: ensaio nas sete instalações de treinamento"
description: "Ensaio de 20/11/2026 de anotar, marcar, atribuir, comentar, acompanhar, documento salvar, bloco incluir e anexar em UnB, PEN1-4, UFRJ e GO; seis promovidas para produção, comentar e deixar de acompanhar seguem restritas"
---

# Promoção de seis escritas — 20/11/2026

**Regra:** a aprovação para produção exige ensaio em todas as instalações de
treinamento. Um bloqueio alheio ao IES fica `NOT_RUN` e não impede a
aprovação (`src/origem_de_treinamento.rs`).

**Contas:** as públicas da [tabela autorizada](../interfaces/credenciais-de-treinamento.md).
As sessões ficaram em pastas `0700` com arquivos `0600`, fora do repositório.

**Execução:** binário release da branch `claude/relaxed-babbage-v0nffs` com
as correções do `atribuir` (302 com `id_procedimento` repetido) e do `bloco
incluir` (número de bloco único), que estão na mesma PR. A UnB foi
ensaiada diretamente. As outras seis foram ensaiadas por agentes paralelos
seguindo um roteiro único, cada um com a própria sessão. A rede foi
direta, sem proxy de captura.

Cada escrita seguiu: prévia, um único `--confirmar` e releitura
independente. Nenhum `--confirmar` que falhou depois do POST foi repetido.
As repetições registradas (abaixo) foram recusas **antes** do POST, e
houve conferência de que nada tinha sido gravado.

## Resultado

| Escrita | UnB | PEN1 | PEN2 | PEN3 | PEN4 | UFRJ | GO | Decisão |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `processo anotar` (gravar e retirar) | PASS | PASS | PASS | PASS | PASS | PASS | PASS | **promovida** |
| `processo marcar` (atribuir e retirar) | PASS | PASS | PASS | PASS | PASS | PASS¹ | PASS | **promovida** |
| `processo atribuir` (atribuir e remover) | PASS² | PASS | PASS | PASS | PASS | PASS | PASS | **promovida** |
| `processo anexar` | PASS³ | PASS | PASS | PASS | PASS | PASS | PASS⁴ | **promovida** |
| `documento salvar` | PASS | PASS | PASS | PASS | PASS | PASS | PASS | **promovida** |
| `bloco incluir --documento` | PASS | PASS | PASS | PASS | NOT_RUN⁵ | PASS | PASS | **promovida** |
| `processo acompanhar` (gravar) | PASS | PASS | PASS | PASS | PASS | PASS | PASS | já não tinha guarda |
| `processo acompanhar --deixar` | FAIL | FAIL | FAIL | FAIL | FAIL | FAIL | FAIL | restrita |
| `processo comentar` | FAIL | FAIL | FAIL | FAIL | FAIL | FAIL | FAIL | restrita |

1. Na UFRJ, a unidade FCC não oferece os marcadores das cassettes. O ensaio
   foi feito na CAXIAS, cujo catálogo a cassette `marcador-escritas-ufrj`
   publica (`46#Baixa prioridade`), num processo próprio. Depois a sessão
   voltou para a FCC.
2. Na UnB, o binário da `main` deu `NaoConfirmadoNaAtribuicao` com
   `atribuicao_relida: aluno2`, num processo recém-criado e sem atribuição.
   A causa é o 302 com `id_procedimento` repetido. Com a correção, atribuir e
   remover confirmaram com saída 0.
3. Na UnB, com o processo anexado **vazio**, a prévia publicou
   `documentos_assinados: true` (verdade vácua), e o POST foi recusado pelo
   SEI (`NaoConfirmado(RespostaDoPost)`). A releitura mostrou que nada foi
   anexado. Depois de criar e assinar um despacho no anexado, uma nova
   prévia e um único `--confirmar` deram saída 0.
4. Na GO, o primeiro `processo criar --confirmar` recusou antes do POST com
   `Fotografia(Caixa(TabelaAusente { categoria: Recebidos }))`: o Controle
   estava na visão detalhada. Depois de `unidade recebidos`, a criação e o
   `anexar` passaram.
5. A unidade TESTE da PEN4 não tem bloco de assinatura (`total_declarado: 0`),
   e o IES não cria blocos.

## Falhas que mantêm as restrições

- **`comentar`:** num processo sem comentários, a lista redireciona para o
  cadastro (302). A leitura recusa com `RedirecionamentoNaoComprovado` e a
  prévia com `TelaDaLista(Redirecionada)`, nas sete instalações. O primeiro
  comentário de um processo nunca é gravável. É a lacuna já declarada em
  `contracts/cli/processo-comentarios.md`.
- **`acompanhar --deixar --confirmar`:** recusa antes do POST com
  `ExclusaoNaoOferecida` nas sete. O leitor de
  `src/processo_acompanhar/exclusao.rs` não reconhece o
  `frmGerenciarAcompanhamento` real. A prévia de `--deixar` sai 0 e não
  antecipa a recusa.

## Incoerências observadas (sem bloquear a promoção)

- A prévia de `marcar` publica `retirada_possivel: false` mesmo com o
  marcador aplicado. Só o confirmar de retirada diz `true`.
- A idempotência sai 1 em `atribuir` (`AtribuicaoJaAplicada`) e `anexar`
  (`ProcessoJaAnexado`), e 0 com `nada_a_fazer` em `marcar`, `acompanhar` e
  `bloco`.
- `documento criar` sem `--nivel`: a prévia sai 0 e o `--confirmar` recusa
  com `NivelNaoInformado`.
- `processo criar` exige a visão resumida do Controle, mas `unidade
  listar-processos` deixa a sessão na detalhada (GO; transitório na PEN2).
- `acompanhar`: `ja_estava` e `id_acompanhamento` divergem entre a prévia de
  gravar e a de `--deixar`.
- `documento assinar` publica `unidade: null`.
- `documento salvar`: `trecho_divergente` parece contar linhas, não bytes.
- O login do `atribuir` diferencia maiúsculas (`ORGAO4`, não `orgao4`, na PEN4).

## Processos e resíduo

| Instalação | Unidade | A | B | Documentos |
| --- | --- | --- | --- | --- |
| UnB 4.0.9 | Unidade1 | `23106.002153/2026-56` | `23106.002154/2026-09` | `0012660` (A, bloco 2503), `0012661` (B, assinado) |
| PEN1 | TESTE | `043509/2026-94` | `043510/2026-19` | `0012531` (A, bloco 61), `0012532` (B, assinado) |
| PEN2 | TESTE | `98290.000150/2026-47` | `98290.000151/2026-91` | `0002171` (A, bloco 7), `0002172` (B, assinado) |
| PEN3 | TESTE | `98390.000101/2026-86` | `98390.000102/2026-21` | `0001072` (A, bloco 2), `0001073` (B, assinado) |
| PEN4 | TESTE | `98490.000125/2026-15` | `98490.000126/2026-51` | `0000929` (A), `0000930` (B, assinado) |
| UFRJ | FCC (+ CAXIAS) | `99079.901804/2026-29` | `99079.901805/2026-73` | `0003140` (A, bloco 274), `0003141` (B, assinado); C `99079.901806/2026-18` na CAXIAS |
| GO 5.0.3 | SEAD/CDE-09516 | `202600005006025`⁶ / `202600005006542` | `202600005006543` | `8447940` (bloco 275346), `8447941` (B, assinado) |

6. A GO usou, para as cinco primeiras escritas, o processo
   `202600005006025`, criado pelo IES em 10/11. A anexação foi feita entre os
   novos `…6542` e `…6543`.

Em todas as instalações:
- B ficou anexado a A, e o despacho de B ficou assinado.
- O acompanhamento especial de A ficou ativo, porque `--deixar` falhou.
- Os despachos de A ficaram com a versão 2 contendo "texto salvo pela CLI".
- Anotação, marcador e atribuição foram desfeitos. Fica só o andamento.

Cerca de 175 a 230 requisições por instalação, contando login e criação dos
processos. Não houve 403, WAF nem captcha.
