---
type: CommandContract
title: Relacionar e desfazer relações de processos
operation: processo.relacionar
command: ies processo relacionar NUP --com NUP [--remover] [--confirmar]
effect: write-process-state
implementation: partial
---

# Relacionar processos (#258)

Operação de relacionamento entre processos e remoção de relacionamentos
existentes. A confirmação (--confirmar) deriva um único POST da tela servida,
não segue 302 e exige releitura independente da lista de relacionamentos
para comprovar o efeito. Efeito incerto sai com código 3.

Abertura com a unidade da sessão exigida: a barra do nó PROCESSO fornece
procedimento_relacionar como URL assinada. GET da tela de relacionamentos,
um formulário frmProcedimentoListarRelacionamentos e uma tabela com summary
Lista de Processos Relacionados. Só os links das linhas da tabela comprovam
vínculos. Uma linha repetida ou uma tabela ausente/repetida é recusa.

**Adicionar relacionamento:** a consulta AJAX protocolo_RI1023 declarada na tela
recebe idProcedimento com o destino informado. O XML deve publicar
IdProcedimento decimal e ProtocoloProcedimentoFormatado, ambos únicos.
Não relaciona consigo mesmo. Id já presente na tabela retorna sem_efeito true,
sem enviar POST de escrita. Com --confirmar e sem_efeito false, submete o
formulário servido com sbmAdicionar e confere a inclusão em nova leitura
da lista. Orçamento a frio: 7 requisições (abertura 3 + tela + consulta + POST + releitura);
abertura indexada: 5 requisições.

**Remover relacionamento:** encontra a linha pelo NUP informado e exige
acaoRemover('id'); na mesma linha. Sem relacionamento prévio, sem_efeito true,
sem enviar POST de escrita. Com --confirmar e sem_efeito false, extrai a URL
de procedimento_excluir_relacionamento declarada em acaoRemover, submete
o formulário com hdnInfraItemId e confere a exclusão em nova leitura da lista.
Orçamento a frio: 6 requisições (abertura 3 + tela + POST + releitura);
abertura indexada: 4 requisições.

Campos de sucesso JSON: nup, id_procedimento, destino, id_destino (null
quando não observado), remover, confirmado, alterado, sem_efeito, origem web
e requisicoes. Nunca expor URL, infra_hash, cookie, senha ou argumento secreto.
Exit 0 sucesso, 1 recusa, 2 uso inválido, 3 efeito incerto.

## Evidência

- Todos @323eb01: src/todos/processos.py::relacionar,
  tests/test_relacionar_010.py, tests/test_processo_relacionar_remover_010.py,
  tests/paginas_reais/procedimento_relacionar.html.
- Cassette PEN1 processo-relacionar-remover-pen1: 25 interações agregadas;
  passos 13 (GET relacionar), 14 (POST excluir) e 15 (releitura 200).
- Testes sintéticos src/processo_relacionar/tests.rs e tests_escrita.rs.
- RED local executado; CI a verificar. Live PEN1/UnB NOT_RUN (0 pedidos);
  escrita em treinamento autorizada.

## Subcódigos tipados da fatia

| Subcódigo | Condição observável ou regra |
| --- | --- |
| `confirmacao_de_relacionamento_nao_validada` | `--confirmar` recusado quando a escrita ainda não estiver habilitada |
| `relacionar_nao_publicado_na_barra` | A barra do processo não publica `procedimento_relacionar` |
| `links_de_relacionar_repetidos_na_barra` | Mais de um link de relacionar na barra do nó PROCESSO |
| `endereco_de_relacionar_nao_reconhecido` | O link da barra não tem a forma observada de `procedimento_relacionar` |
| `endereco_de_relacionar_de_outro_processo` | O link da barra aponta para outro `id_procedimento` |
| `tela_de_relacionamentos_redirecionada` | O GET da tela de relacionamentos respondeu com redirecionamento |
| `relacionamento_consigo_mesmo` | O destino informado é o próprio processo |
| `remocao_de_relacionamento_nao_oferecida` | A linha do relacionamento existe, mas não traz `acaoRemover('id')` |
| `acao_excluir_relacionamento_nao_declarada` | A ação acaoRemover não contém a URL de procedimento_excluir_relacionamento |
| `efeito_incerto` | A escrita foi submetida, mas a releitura independente não comprovou o efeito esperado |

Outros subcódigos já definidos pelo transporte, abertura e primitivo de
formulário conservam sua identidade. Recusas não autorizam retry.
