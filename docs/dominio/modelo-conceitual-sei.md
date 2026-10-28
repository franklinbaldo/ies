---
type: Guide
title: Modelo conceitual do SEI
description: Vocabulário operacional, identidades, vínculos e invariantes do domínio
---

# Modelo conceitual do SEI

Este é o vocabulário de referência do IES. Não é esquema de banco,
cadastro paralelo, nem obrigação de criar uma struct para cada item.
Os modelos Rust nascem quando um contrato de comando precisa deles.
As interfaces REST, SOAP e Web são meios de obter fatos, não o domínio.

## Como usar na implementação

1. Identifique o objeto, o ID interno e o identificador exibido; nunca
   trate números de sistemas ou instalações diferentes como universais.
2. Identifique o contexto: instalação, identidade, unidade e permissão.
3. Separe propriedades relativamente estáveis, estados mutáveis,
   eventos históricos e observações da interface.
4. Implemente só propriedades demonstradas pela fonte usada; ausência
   de campo não equivale a valor falso.
5. Registre no contrato o que é consultável, o que é alterável e como
   reconciliar uma escrita com resultado incerto.
6. Quando a fonte não permite distinguir conceitos, preserve a
   incerteza em vez de construir uma equivalência.

## Ambiente e organização

| Conceito | Significado e limites |
| --- | --- |
| Instalação | Implantação SEI identificada por origem HTTPS e caminho-base, não um órgão. |
| Versão SEI | Versão declarada ou confirmada; não garante módulos, permissões ou equivalência. |
| Módulo | Componente opcional/configurado, como WSSEI REST; suporte não é inferido da versão. |
| Capacidade | Suporte a uma operação por módulo, instalação, identidade e unidade; disponibilidade, autorização e equivalência são perguntas diferentes. |
| Órgão | Entidade administrativa associada a unidades; uma instalação pode atender mais de um órgão. |
| Unidade | Unidade organizacional com identidade própria dentro do escopo da instalação. Não é a instalação nem a sessão. |

Capacidade **verificada** significa teste autorizado no escopo descrito;
capacidade **sugerida** por mesma versão não autoriza execução automática;
**desconhecida** permanece desconhecida. Evitar percentuais e atributos
globais do tipo `rest=true` para afirmar operação permitida.

## Identidade, acesso e sessão

| Conceito | Significado e limites |
| --- | --- |
| Pessoa | Pessoa física ou jurídica referenciada em papéis distintos; não equivale a usuário autenticado. |
| Usuário | Identidade de acesso na instalação; distinguir usuário interno e externo. |
| Vínculo usuário-unidade | Relação que permite atuação em unidade conforme cadastro e perfil. |
| Perfil/permissão | Autorização que limita ações, inclusive por unidade e módulo. |
| Sessão | Contexto autenticado e transitório, com unidade ativa quando aplicável. |
| Interessado | Participante vinculado ao processo; não pressupõe conta de usuário. |
| Acesso externo | Concessão específica a usuário externo; não é a assinatura em si. |

A mesma pessoa pode aparecer como interessado, usuário ou assinante;
esses papéis não são identificadores intercambiáveis. A unidade ativa
não pode ser inferida somente pelo host. Tokens, cookies e links assinados
não entram em documentos OKF nem no modelo público.

## Processo e sua atuação nas unidades

| Conceito | Significado e limites |
| --- | --- |
| Processo | Objeto administrativo com ID interno e número exibido (NUP quando aplicável). |
| Tipo de processo | Classificação administrativa, separada da identidade do processo. |
| Atuação processo-unidade | Situação contextual da unidade perante o processo; não é um booleano global do processo. |
| Atribuição | Responsável por tratamento na unidade; não substitui vínculo organizacional. |
| Andamento | Entidade de registro histórico do processo, com data/hora, unidade, usuário e descrição quando disponíveis; não necessariamente envio. |
| Marcador/anotação | Organização da atividade; não determina situação global. |
| Nível de acesso | Regime de publicidade/restrição do processo ou documento, observado no contexto autorizado. |

**Invariantes confirmados no manual:** processo pode estar aberto em mais
de uma unidade; concluir numa não conclui nas demais. Envio pode ter
múltiplas destinatárias e, sem a opção de manter aberto, pode concluir
a atuação da remetente. Reabertura em unidade onde já tramitou não
exige necessariamente novo envio e pode atribuir o processo ao usuário
que reabriu. Não derivar `processo.aberto: bool` global.

**Não confundir:** aberto/concluído na unidade, aguardando recebimento,
sobrestamento, acesso permitido e atribuição ao usuário. Esses atributos
têm regras e momentos distintos. O indicador `visualizado` da caixa é
contextual; não é uma propriedade imutável do processo.

## Tramitação: questão aberta

Tramitação é um conceito de negócio, ainda **não** uma entidade Rust
aprovada. **Andamento já é uma entidade conceitual aprovada**: o manual
identifica registros individuais com data/hora, unidade, usuário e descrição.
A operação de atualizar andamento insere um registro textual sem permitir
edição ou exclusão posterior; correções são lançadas como novos registros.
O identificador técnico de cada andamento ainda precisa ser comprovado.
Não inventar ID a partir de data/hora ou posição da tabela. Candidatos a separar após exame de cassettes e manuais:

- **Envio**: ato iniciado pela remetente, potencialmente com vários
  destinatários e a opção de manter aberto.
- **Recebimento**: acontecimento registrado na destinatária.
- **Andamento**: evento do histórico, que pode registrar muitos outros
  atos além de envio e recebimento.
- **Atuação processo-unidade**: visão da situação em determinada unidade.

Investigar, antes de definir identificadores/cardinalidades: como o
SEI representa um envio com várias destinatárias; quais eventos
comprovam recebimento; se há ID estável da remessa; como são expostos
reabertura, conclusão, sobrestamento e processos sigilosos. Não
presumir que uma lista de andamentos seja uma trilha linear de
transferências, nem misturar fluxos internos com Tramita.GOV.BR.

## Documento, assinatura e blocos

| Conceito | Significado e limites |
| --- | --- |
| Documento | Item do processo, interno ou externo, com ID, número exibido, tipo e conteúdo conforme autorização. |
| Versão de documento | Alteração/revisão de conteúdo, quando a interface comprovar sua identidade e disponibilidade. |
| Assinatura | Ato vinculado a documento e assinante; documento admite múltiplas assinaturas. |
| Cargo/função de assinatura | Opção configurada para a unidade; não pressupor cargo global do usuário. |
| Bloco de assinatura | Recurso para disponibilizar documentos para assinatura por outras unidades; não é a assinatura nem envio do processo. |
| Autenticidade/verificação | Conferência de assinatura/documento distinta de simplesmente exibir um registro de assinatura. |

O manual documenta assinatura por senha ou certificado e rodapé com
informações de assinatura. Documento assinado pode mudar de
editabilidade quando acessado por usuário de outra unidade. A
funcionalidade de assinatura descrita no manual administrativo é
voltada a documentos internos. Não inferir validade criptográfica
apenas do ícone, rodapé ou registro visual. Não presumir que assinatura
é atributo booleano: uma lista de atos, com contexto próprio, é mais
adequada quando houver dados confirmados.

## Tipos no runtime: recorte atual e futuro

- **Existentes**: `Instalacao`, `VersaoSei`, `IdProcedimento`,
  `Processo` mínimo e `Caixa`. A leitura da caixa não deve ser
  ampliada para transportar toda a ontologia.
- **Ao implementar operações**: IDs de unidade, usuário e documento,
  `Documento`, `Assinatura` e situação da atuação na unidade,
  conforme campos comprovados e contratos respectivos.
- **Somente conceituais por enquanto**: Pessoa, órgão, módulo,
  perfil, andamento (entidade aprovada, sem estrutura Rust), envio, recebimento, bloco de assinatura e
  classificações ainda não necessárias aos comandos.

Novos tipos devem distinguir IDs internos, números visíveis e
identidade de instalação. `Option` representa dado não obtido; não
deve mascarar ausência de autorização nem incerteza de escrita.
Não criar ORM, banco, pesquisa ou camada genérica de observação.

## Cenários de aceite para futuros testes

1. Duas instalações de mesma versão podem divergir no WSSEI REST.
2. Uma instalação pode servir vários órgãos e uma sessão pode trocar
   unidade sem mudar instalação.
3. O mesmo processo está aberto em A e concluído em B sem contradição.
4. Um envio com dois destinatários não elimina a atuação da remetente
   quando o usuário escolheu manter aberto.
5. Um documento contém mais de uma assinatura; cargo/função é
   interpretado no contexto da unidade.
6. Um usuário pode ser interessado sem possuir login e um assinante
   não se identifica apenas pelo nome visível.
7. Falta de evidência de uma capacidade nunca habilita execução;
   resultado incerto de escrita nunca autoriza repetição automática.
8. O replay da caixa segue idêntico: nenhuma mudança no JSON v1
   é exigida por este documento.

## Fontes e próximos levantamentos

- [Operações básicas com processos](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Operacoes_basicas_com_processos.html)
- [Operações básicas com documentos](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Operacoes_basicas_com_documentos.html)
- [Blocos e blocos de assinatura](https://manuais.processoeletronico.gov.br/pt-br/latest/SEI/Blocos.html)
- [Assinaturas das unidades](https://manuais.processoeletronico.gov.br/pt-br/latest/SEIADM/Administracao_sei/Man_Adm_SEI_04-Administracao_SEI-01_Assinatura_das_Unidades.html)
- [Acesso por instalação](../interfaces/acesso-por-instalacao.md)
- [Arquitetura de transportes](../arquitetura/0001-rotas-por-evidencia.md)
- [Conhecimento empírico de telas](../como-o-sei-funciona/README.md)
- [Contrato da CLI](../../contracts/cli.md)

Próxima investigação: comparar o conceito de tramitação dos manuais
com cassettes de envio, recebimento, conclusão e reabertura; extrair
IDs e cardinalidades sem presumir o resultado. Em paralelo, mapear
usuário/unidade e documento/assinatura contra contratos REST/SOAP
oficiais e evidências Web, mantendo divergências explícitas.
